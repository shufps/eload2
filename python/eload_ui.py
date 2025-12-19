import json
import time
import logging
import yaml
import streamlit as st
from streamlit_autorefresh import st_autorefresh

# Import your ELoad class from your existing module/file.
# Option A: put your ELoad code into eload.py and do: from eload import ELoad
# Option B: paste ELoad class into this file above.
from eload import ELoad  # <- adjust to your filename

def setup_logging():
    logger = logging.getLogger()
    if logger.handlers:
        return
    logger.setLevel(logging.INFO)
    h = logging.StreamHandler()
    h.setFormatter(logging.Formatter("%(asctime)s - %(levelname)s - %(message)s"))
    logger.addHandler(h)

@st.cache_resource
def load_config(path: str):
    with open(path, "r") as f:
        return yaml.safe_load(f)

@st.cache_resource
def create_eload(cfg):
    # Keep a single ELoad instance alive across reruns
    return ELoad(cfg)

def main():
    setup_logging()
    st.set_page_config(page_title="mini ELoad", layout="wide")

    cfg = load_config("config.yml")
    eload = create_eload(cfg)

    st.title("mini ELoad")

    # Auto refresh UI (and state reading) every 500ms
    st_autorefresh(interval=500, key="refresh")

    col_ctrl, col_state = st.columns([1, 2])

    with col_ctrl:
        st.subheader("Control")

        # Read current control values for defaults
        ctrl = eload.get_control()

        # Your code uses: sdn=True means shutdown
        # UI uses: load_enabled=True means NOT shutdown
        load_enabled_default = not bool(ctrl.get("sdn", False))

        load_enabled = st.toggle("Load aktiv", value=load_enabled_default)
        pwm = st.slider("PWM", min_value=0.0, max_value=1.0, value=float(ctrl.get("pwm", 0.0)), step=0.01)
        current = st.number_input("Strom (A)", min_value=0.0, max_value=200.0, value=float(ctrl.get("dac", 0.0)), step=0.1)

        # NOTE: your get_control() returns dac raw, not current.
        # We'll store the user's desired current separately in session_state.
        if "desired_current" not in st.session_state:
            st.session_state.desired_current = 0.0

        st.session_state.desired_current = st.number_input(
            "Sollstrom (A)", min_value=0.0, max_value=200.0,
            value=float(st.session_state.desired_current), step=0.1
        )

        c1, c2, c3 = st.columns(3)
        with c1:
            if st.button("Apply", use_container_width=True):
                # Apply in a safe order:
                # 1) PWM
                # 2) Enable/disable load
                # 3) Current (only if enabled)
                eload.set_pwm(pwm)
                eload.set_shutdown(not load_enabled)  # invert UI -> device meaning
                if load_enabled:
                    eload.set_current(float(st.session_state.desired_current))
                st.toast("Applied", icon="✅")

        with c2:
            if st.button("AUS (Shutdown)", use_container_width=True):
                eload.shutdown()
                st.toast("Shutdown sent", icon="🛑")

        with c3:
            if st.button("Nur Refresh", use_container_width=True):
                st.rerun()

        st.caption("Hinweis: `sdn=True` wird als Shutdown interpretiert (wie in deiner shutdown()-Methode).")

    with col_state:
        st.subheader("State")

        try:
            state = eload.get_state()
        except Exception as e:
            st.error(f"State read failed: {e}")
            st.stop()

        amps = state.get('ch0', 0.0) + state.get('ch1', 0.0) + state.get('ch2', 0.0) + state.get('ch3', 0.0)
        volts = state.get('v', 0.0)

        # Metrics
        m1, m2, m3, m4 = st.columns(4)
        m1.metric("V", f"{state.get('v', 0.0):.3f}")
        m2.metric("P", f"{volts * amps:.2f} W")
        m3.metric("Temp", f"{state.get('temp', 0.0):.2f} °C")
        m4.metric("SDN", "TRUE" if state.get("sdn") else "FALSE")

        cA, cB, cC, cD = st.columns(4)
        cA.metric("CH0 (A)", f"{state.get('ch0', 0.0):.3f}")
        cB.metric("CH1 (A)", f"{state.get('ch1', 0.0):.3f}")
        cC.metric("CH2 (A)", f"{state.get('ch2', 0.0):.3f}")
        cD.metric("CH3 (A)", f"{state.get('ch3', 0.0):.3f}")

        st.subheader("Raw JSON")
        st.code(json.dumps({"state": state, "control": eload.get_control()}, indent=2), language="json")

if __name__ == "__main__":
    main()

