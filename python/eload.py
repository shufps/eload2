
import serial
import time
import logging
import random
import copy
import os
import math
import yaml
import json
import keyboard

import coms_pb2
import binascii

import threading

from influx import Influx, Stats

VREFINT = 1.224


class Control:
    def __init__(self, sdn, pwm, dac):
        self.sdn = sdn
        self.pwm = pwm
        self.dac0 = dac
        self.dac1 = dac
        self.dac2 = dac
        self.dac3 = dac
        pass

    def to_dict(self):
        return {
            'sdn': self.sdn,
            'pwm': self.pwm,
            'dac0': self.dac0,
            'dac1': self.dac1,
            'dac2': self.dac2,
            'dac3': self.dac3,
        }

class State:
    def __init__(self):
        self.ch0 = 0
        self.ch1 = 0
        self.ch2 = 0
        self.ch3 = 0
        self.cal = 0
        self.v = 0
        self.temp = 0
        self.sdn = False

    def to_dict(self):
        return {
            'ch0': self.ch0,
            'ch1': self.ch1,
            'ch2': self.ch2,
            'ch3': self.ch3,
            'cal': self.cal,
            'v': self.v,
            'temp': self.temp,
            'sdn': self.sdn,
        }

class ELoad:
    def __init__(self, config):
        self.config = config

        self.reqid = 0
        self.serial_port_ctrl_lock = threading.Lock()

        self.control = Control(True, 0, 0)
        self.state = State()

        # Initialize serial communication
        self._serial_port_ctrl = serial.Serial(
            port=self.config['serial_port'],  # For GPIO serial communication use /dev/ttyS0
            baudrate=115200,    # Set baud rate to 115200
            bytesize=serial.EIGHTBITS,     # Number of data bits
            parity=serial.PARITY_NONE,     # No parity
            stopbits=serial.STOPBITS_ONE,  # Number of stop bits
            timeout=1                      # Set a read timeout
        )

    def set_pwm(self, pwm):
        self.control.pwm = pwm
        self._send_control()

    def _request(self, op, params):
        request = coms_pb2.QRequest()
        request.id = self.reqid  # Set a unique ID for the request
        request.op = op

        if params is not None:
            request.data = params.SerializeToString()
        else:
            request.data = b'0x00'
        request.data = bytes([len(request.data)]) + request.data

        serialized_request = request.SerializeToString()
        serialized_request = bytes([len(serialized_request)]) + serialized_request

        logging.debug("-> %s", binascii.hexlify(serialized_request).decode('utf8'))

        self._serial_port_ctrl.write(serialized_request)

        response_len = self._serial_port_ctrl.read()
        logging.debug(f"rx len: {response_len}")
        if len(response_len) == 1 and response_len[0] == 0:
            self.reqid += 1
            return coms_pb2.QResponse()

        response_data = self._serial_port_ctrl.read(response_len[0])

        logging.debug("<- %s", binascii.hexlify(response_data).decode('utf8'))

        response = coms_pb2.QResponse()
        response.ParseFromString(response_data)

        if response.id != self.reqid:
            logging.error(f"request and response IDs mismatch! {response.id} vs {self.reqid}")

        self.reqid += 1
        return response

    def _current_to_dac(self, current):
        r_sense = 0.004 # 4mR
        r1 = 31600.0 # 31.6k
        r2 = 1000.0 # 1k
        num_parallel = 4.0 # 4 mosfets
        correct = 1.136363

        voltage = current * (r1 + r2) / r2 * r_sense
        dac_value = (int(voltage / 3.3 * 65536.0 / num_parallel * correct)) & 0xffff

        logging.info(f"current: {current}, voltage_millis: {voltage}, dac_value: {dac_value}");
        return dac_value

    def set_shutdown(self, shutdown):
        self.control.sdn = 1 if shutdown else 0
        self._send_control()

    def set_current(self, current):
        self.control.dac0 = self._current_to_dac(current)
        self.control.dac1 = self._current_to_dac(current)
        self.control.dac2 = self._current_to_dac(current)
        self.control.dac3 = self._current_to_dac(current)
        self._send_control()

    def _adc_to_current(self, v):
        r_sense = 0.004 # 4mR
        r1 = 31600.0 # 31.6k
        r2 = 1000.0 # 1k
        return v / r_sense / (1.0 + r1 / r2)

    def _calc_ampere(self, sample, cal):
        voltage = float(sample) * VREFINT / float(cal);
        return self._adc_to_current(voltage)

    def _calc_voltage(self, sample, cal):
        r1 = 33000.0
        r2 = 10000.0
        voltage = float(sample) * VREFINT / float(cal) * (r1 + r2) / r2;
        return voltage

    def _receive_state(self):
        with self.serial_port_ctrl_lock:
            resp = self._request(2, None)
            if resp is None or resp.error != 0:
                raise Exception("failed reading status!")

            status = coms_pb2.QState()
            status.ParseFromString(resp.data[1:])

            self.state.ch0 = self._calc_ampere(status.ch0, status.cal)
            self.state.ch1 = self._calc_ampere(status.ch1, status.cal)
            self.state.ch2 = self._calc_ampere(status.ch2, status.cal)
            self.state.ch3 = self._calc_ampere(status.ch3, status.cal)
            self.state.cal = status.cal
            self.state.v = self._calc_voltage(status.v, status.cal)
            self.state.p = self.state.v * (self.state.ch0 + self.state.ch1 + self.state.ch2 + self.state.ch3)
            self.state.temp = status.temp * 0.0625
            self.state.sdn = True if status.sdn == 1 else False

    def get_state(self):
        self._receive_state()
        return self.state.to_dict()

    def get_control(self):
        return self.control.to_dict()

    def _send_control(self):
        with self.serial_port_ctrl_lock:
            qcontrol = coms_pb2.QControl()
            qcontrol.sdn = 1 if self.control.sdn else 0
            qcontrol.pwm = int(100.0 * float(self.control.pwm))
            qcontrol.dac0 = self.control.dac0
            qcontrol.dac1 = self.control.dac1
            qcontrol.dac2 = self.control.dac2
            qcontrol.dac3 = self.control.dac3
            if self._request(1, qcontrol).error != 0:
                raise Exception("error send_control")

    def shutdown(self):
        logging.info("shutdown ...")
        self.control.dac0 = 0
        self.control.dac1 = 0
        self.control.dac2 = 0
        self.control.dac3 = 0
        self.control.sdn = True
        self.control.pwm = 0
        self._send_control()

    def serial_port(self):
        return self._serial_port_asic

