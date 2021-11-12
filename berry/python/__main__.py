import sys

lib_path = "/home/pi/university/rust-iot-project/berry/python/max30102"
sys.path.insert(1, lib_path)

import threading
import Adafruit_DHT
import RPi.GPIO as GPIO

import time
import numpy as np
import max30102
import hrcalc

from time import sleep


# BOARD numbering
DHT_SENSOR = Adafruit_DHT.DHT11
LED_PIN = 35
BUZZ_PIN = 37

# GPIO numbering
DHT_PIN = 10


GPIO.setwarnings(False)
GPIO.setmode(GPIO.BOARD)
GPIO.setup(BUZZ_PIN, GPIO.OUT)
GPIO.setup(LED_PIN, GPIO.OUT)


def main(argv):
    while True:
        GPIO.output(BUZZ_PIN, GPIO.LOW)
        GPIO.output(LED_PIN, GPIO.LOW)

        avg_bpm, avg_oxi = read_bpm(5)
        avg_tmp = read_temperature(3)

        buzz(avg_bpm, avg_oxi, avg_tmp, 5)
        th = threading.Thread(target=flush_max30102(5))
        th.start()

        GPIO.output(LED_PIN, GPIO.LOW)

        sleep(1)
        th.join()


MAX_BPM = 100
MIN_BPM = 60
MAX_TMP = 37.5
MIN_OXI = 95


def buzz(bpm, oxi, tmp, span):
    abnormal_bpm = bpm < MIN_BPM or bpm > MAX_BPM
    high_tmp = tmp > MAX_TMP
    low_oxi = oxi < MIN_OXI

    normal_indicators = (not low_oxi) and (not high_tmp) and (not abnormal_bpm)

    freq = 1

    start = time.time()
    current = start

    if not normal_indicators:
        if abnormal_bpm:
            freq = freq - 0.25
        if high_tmp:
            freq = freq - 0.25
        if low_oxi:
            freq = freq - 0.25
        while current - start <= span:
            beep(freq)
            current = time.time()


def beep(freq):
    GPIO.output(BUZZ_PIN, GPIO.HIGH)
    sleep(freq)
    GPIO.output(BUZZ_PIN, GPIO.LOW)
    sleep(freq)


def flush_max30102(n):
    m = max30102.MAX30102()

    for _ in range(0, n):
        m.read_sequential(150)

    m.reset()
    m.shutdown()


def read_bpm(n):
    print("oximetry starting...")
    m = max30102.MAX30102()

    bpms = []
    spo2s = []

    no_finger = True

    while True:
        red, ir = m.read_sequential(150)

        if avg(ir) < 50000 or avg(red) < 50000:
            print("No finger ...")
            sleep(0.5)
        else:
            if no_finger:
                beep(0.25)
                GPIO.output(LED_PIN, GPIO.HIGH)
                no_finger = False

            bpm, valid_bpm, spo2, valid_spo2 = hrcalc.calc_hr_and_spo2(ir, red)

            valid_measurements = valid_bpm and valid_spo2
            human_range = bpm >= 30 and bpm <= 220

            if valid_measurements and human_range:
                bpms.append(bpm)
                spo2s.append(spo2)

            if len(bpms) > n and len(spo2s) > n:
                return (avg(bpms), avg(spo2s))


def read_temperature(n):
    print("temperature starting...")
    readings = []

    for _ in range(0, n):
        _, temperature = Adafruit_DHT.read_retry(DHT_SENSOR, DHT_PIN)
        if temperature is not None:
            readings.append(temperature)
        else:
            print("Sensor failure. Check wiring")

    return avg(readings)


def avg(ls):
    if len(ls) < 1:
        return 0
    else:
        return np.mean(ls)


if __name__ == "__main__":
    main(sys.argv[1:])
