from influxdb_client import Point
from influxdb_client.client.write_api import SYNCHRONOUS
from influxdb_client import InfluxDBClient, Point, WriteOptions, WritePrecision
from datetime import datetime
import pytz
import logging
import threading
import json
import time

class Stats:
    def __init__(self):
        self.temp = 25.0
        self.ch0 = 0.0
        self.ch1 = 0.0
        self.ch2 = 0.0
        self.ch3 = 0.0
        self.v = 0.0
        self.p = 0.0

        self.lock = threading.Lock()

class Influx:
    def __init__(self, config, stats, stats_name):
        # InfluxDB settings (replace with your own settings)
        self.host = config['host']
        self.port = config['port']
        self.token = config['token']
        self.org = config['org']
        self.bucket = config['bucket']
        self.stats_name = stats_name
        self.client = None
        self.tz = pytz.timezone(config['timezone'])
        self.stats = stats
        self.stop_event = threading.Event()
        self.connect()

    def start(self):
        self.submit_thread = threading.Thread(target=self._submit_thread)
        self.submit_thread.start()

    def _submit_thread(self):
        while not self.stop_event.is_set():
            if not self.client:
                time.sleep(10)
                continue

            with self.stats.lock:
                point = Point(f"{ self.stats_name }").time(datetime.now(self.tz), WritePrecision.NS) \
                    .field("temperature", float(self.stats.temp)) \
                    .field("ch0", float(self.stats.ch0)) \
                    .field("ch1", float(self.stats.ch1)) \
                    .field("ch2", float(self.stats.ch2)) \
                    .field("ch3", float(self.stats.ch3)) \
                    .field("v", float(self.stats.v)) \
                    .field("p", float(self.stats.p))

            try:
                write_api = self.client.write_api(write_options=SYNCHRONOUS)
                write_api.write(bucket=self.bucket, org=self.org, record=point)
                logging.debug("influx data written: %s", point.to_line_protocol())
            except Exception as e:
                logging.error("writing to influx failed: %s", e)

            time.sleep(15)

    def connect(self):
        # Connect to InfluxDB
        try:
            self.client = InfluxDBClient(url=f"http://{self.host}:{self.port}", token=self.token, org=self.org)
            self.create_bucket(self.bucket)
        except Exception as e:
            logging.error("connecting influx failed: %s", e)

    def bucket_exists(self, bucket_name):
        # List all buckets
        buckets = self.client.buckets_api().find_buckets().buckets

        # Check if the specified bucket is in the list
        for bucket in buckets:
            if bucket.name == bucket_name:
                return True
        return False

    def create_bucket(self, bucket_name):
        if self.bucket_exists(bucket_name):
            return

        self.client.buckets_api().create_bucket(bucket_name = bucket_name)


    def close(self):
        # Close the connection
        if self.client:
            self.client.close()
