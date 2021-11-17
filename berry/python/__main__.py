import sys

lib_path = "/home/pi/university/rust-iot-project/berry/python/max30102"
sys.path.insert(1, lib_path)

import http.client
import json
import threading
import time
import numpy as np
from time import sleep

import Adafruit_DHT
import RPi.GPIO as GPIO
import hrcalc
import max30102


# BOARD numbering
LED_PIN = 35
BUZZ_PIN = 37

# GPIO numbering
DHT_PIN = 10

# General setup
GPIO.setwarnings(False)
GPIO.setmode(GPIO.BOARD)
GPIO.setup(BUZZ_PIN, GPIO.OUT)
GPIO.setup(LED_PIN, GPIO.OUT)

# Sensors
DHT_SENSOR = Adafruit_DHT.DHT11

def main(argv):
    GPIO.output(LED_PIN, GPIO.LOW)
    GPIO.output(BUZZ_PIN, GPIO.LOW)

    while True:
        MAX30102 = max30102.MAX30102()

        # Take five samples of bpm and oximetry and bind averages to variables.
        avg_bpm, avg_oxi = read_bpm(5, 150, MAX30102)

        # Take three samples of temperature and bind average to variable.
        avg_tmp = read_temperature(1)

        GPIO.output(LED_PIN, GPIO.LOW)
        time.sleep(2)

        # Flush MAX30102 on a different thread to avoid inconsistencies in next iteration.
        th = threading.Thread(target=flush_max30102(5, 150, MAX30102))
        th.start()

        # POST readings to remote server.
        post_req(avg_bpm, avg_oxi, avg_tmp)

        # Send readings to buzz helper to alert user in case of abnormal readings.
        buzz(avg_bpm, avg_oxi, avg_tmp, 5)

        # Turn led down in order to notify user that measurement is done.
        GPIO.output(LED_PIN, GPIO.LOW)

        # Wait for `flush_max30102` to finish.
        th.join()

        time.sleep(1)


# Normal BPM readings should be between 60 and 100 beats per minute when resting.
MAX_BPM = 100
MIN_BPM = 60

# Normal body TEMPERATURE should be below 37.5.
MAX_TMP = 37.5


# Normal OXIMETRY in blood should be greater or equal to 95%.
MIN_OXI = 95


# Given a BPM reading, an OXIMETRY reading, and a TEMPERATURE reading turn BUZZER on
# for a specific time span in seconds only if readings are abnormal.
def buzz(bpm, oxi, tmp, span):
    # BPM are abnormal if less than MIN_BPM or greater than MAX_BPM.
    abnormal_bpm = bpm < MIN_BPM or bpm > MAX_BPM

    # Body TEMPERATURE is considered high if greater than MAX_TMP.
    high_tmp = tmp > MAX_TMP

    # Blood OXIMETRY is considered low if less than 95%.
    low_oxi = oxi < MIN_OXI

    # Indicators are normal if BPM, OXIMETRY and TEMPERATURE are ok.
    normal_indicators = (not low_oxi) and (not high_tmp) and (not abnormal_bpm)

    # Delay between buzzer beeps.
    freq = 1

    start = time.time()
    current = start

    if not normal_indicators:
        # If abnormal BPM reduce beep frequency so that BUZZER buzzes faster.
        if abnormal_bpm:
            freq = freq - 0.25
        # If high TEMPERATURE reduce beep frequency so that BUZZER buzzes faster.
        if high_tmp:
            freq = freq - 0.25
        # If low OXIMETRY reduce beep frequency so that BUZZER buzzes faster.
        if low_oxi:
            freq = freq - 0.25

        # Loop for a specific time span.
        while current - start <= span:
            beep(freq)
            current = time.time()


# Given a frequency in seconds, buzzer beeps.
def beep(freq):
    GPIO.output(BUZZ_PIN, GPIO.HIGH)
    sleep(freq)
    GPIO.output(BUZZ_PIN, GPIO.LOW)
    sleep(freq)


# Flush and reset MAX30102.
def flush_max30102(n, k, mx):
    for _ in range(0, n):
        red, ir = mx.read_sequential(k)

    mx.shutdown()

# Take n samples of BPM and OXIMETRY measurements and return average.
def read_bpm(n, k, mx):
    bpms = []
    spo2s = []

    no_finger = True

    while True:
        # Take 150 readings of both red and infra-red leds via sensor MAX30102.
        red, ir = mx.read_sequential(k)

        # If average readings are below 50000, it means that there is no finger.
        if avg(ir) < 50000 or avg(red) < 50000:
            print("No finger ...")
            sleep(0.5)
        else:
            # Notify user that readings has started by beeping and turning led on.
            if no_finger:
                print("oximetry starting...")

                beep(0.25)
                GPIO.output(LED_PIN, GPIO.HIGH)
                no_finger = False

            # Compute bpm and OXIMETRY (spo2)
            bpm, valid_bpm, spo2, valid_spo2 = hrcalc.calc_hr_and_spo2(ir, red)

            valid_measurements = valid_bpm and valid_spo2
            human_range = bpm >= 30 and bpm <= 220

            # If measurements are both valid and within the human range, push them
            # into the array of samples.
            if valid_measurements and human_range:
                bpms.append(bpm)
                spo2s.append(spo2)

            # If there are n valid samples, return the average BPM and OXIMETRY.
            if len(bpms) > n and len(spo2s) > n:
                return (avg(bpms), avg(spo2s))


# Take n samples of TEMPERATURE measurements and return average.
def read_temperature(n):
    print("temperature starting...")
    readings = []

    for _ in range(0, n):
        # Read temperature via sensor DHT11.
        _, temperature = Adafruit_DHT.read_retry(DHT_SENSOR, DHT_PIN)
        if temperature is not None:
            # Push reading
            readings.append(temperature)
        else:
            print("Sensor failure. Check wiring")

    # Return average.
    return avg(readings)


# Given a list of numbers return its average.
def avg(ls):
    if len(ls) < 1:
        return 0
    else:
        return np.mean(ls)


# Given BPM, OXIMETRY and TEMPERATURE readings, post JSON payload to server.
def post_req(bpm, oxi, tmp):
    # Host where data is sent.
    conn = http.client.HTTPConnection("25e2-2800-e2-e00-739-d131-e230-79ee-593c.ngrok.io:80")
    headers = {"Content-type": "application/json"}

    # JSON payload to be sent to server.
    payload = json.dumps({"bpm": bpm, "temperature": tmp, "oximetry": oxi})

    # Make request to endpoint where data is to be posted.
    conn.request("POST", "/variables", payload, headers)

    # Get response.
    res = conn.getresponse()

    # Close connection.
    conn.close()

    # Log status code.
    print(f"status code {res.code}")


if __name__ == "__main__":
    main(sys.argv[1:])
