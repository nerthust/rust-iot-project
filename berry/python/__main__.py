import sys

lib_path = "/home/pi/university/rust-iot-project/berry/python/max30102"
sys.path.insert(1, lib_path)

import Adafruit_DHT
import RPi.GPIO as GPIO

import time
import numpy as np
import max30102
import hrcalc

from time import sleep


DHT_SENSOR = Adafruit_DHT.DHT11
DHT_PIN = 10

BUZZ_PIN = 26

GPIO.setwarnings(False)
GPIO.setmode(GPIO.BCM)
GPIO.setup(BUZZ_PIN, GPIO.OUT)


def main(argv):
    while True:
        avg_bpm, avg_oxi = read_bpm(10)
        avg_tmp = read_temperature(10)
        buzz(avg_bpm, avg_oxi, avg_tmp, 5)
        sleep(1)


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
    current = start()

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


def read_bpm(n):
    print("oximetry starting...")
    m = max30102.MAX30102()

    bpms = []
    spo2s = []

    while True:
        red, ir = m.read_sequential(125)

        if np.mean(ir) < 50000 or np.mean(red) < 50000:
            print("No finger ...")
            sleep(0.5)
        else:
            beep(0.25)
            bpm, valid_bpm, spo2, valid_spo2 = hrcalc.calc_hr_and_spo2(ir, red)

            valid_measurements = valid_bpm and valid_spo2
            human_range = bpm >= 30 and bpm <= 220

            if valid_measurements and human_range:
                bpms.append(bpm)
                spo2s.append(spo2)

            if len(bpms) > n and len(spo2s) > n:
                return (np.mean(bpms), np.mean(spo2s))


def read_temperature(n):
    readings = []

    for _ in range(0, n):
        _, temperature = Adafruit_DHT.read_retry(DHT_SENSOR, DHT_PIN)

        if temperature is not None:
            readings.append(temperature)
        else:
            print("Sensor failure. Check wiring")

    return np.mean(readings)


if __name__ == "__main__":
    main(sys.argv[1:])
