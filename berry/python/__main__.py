import sys
sys.path.insert(1, '/home/pi/university/rust-iot-project/berry/python/max30102')

import time
import numpy as np
from time import sleep
import max30102
import hrcalc

import Adafruit_DHT
import RPi.GPIO as GPIO


DHT_SENSOR = Adafruit_DHT.DHT11
DHT_PIN = 10

#GPIO.setwarnings(False)
#GPIO.setmode(GPIO.BCM)
#GPIO.setup(4,GPIO.OUT)

#def main(argv):
#    while True: 
#        GPIO.output(4, GPIO.HIGH)
#        sleep(0.5)
#        GPIO.output(4, GPIO.LOW)
#        sleep(0.5)

#    while True:
#        humidity, temperature = Adafruit_DHT.read_retry(DHT_SENSOR, DHT_PIN)
#
#        if humidity is not None and temperature is not None:
#            print("Temp={}C, Humidity={}%".format(temperature, humidity))
#        else:
#            print("Sensor failure. Check wiring")
#
#        time.sleep(2)



def main(argv):
     print('sensor starting...')
     m = max30102.MAX30102()

     bpms = []
     spo2s = []

     while True:
       red, ir = m.read_sequential(150)

       if np.mean(ir) < 50000 or np.mean(red) < 50000:
         print("No finger")
         sleep(0.5)
       else:
         bpm, valid_bpm, spo2, valid_spo2 = hrcalc.calc_hr_and_spo2(ir, red)

         valid_measurements = valid_bpm and valid_spo2
         human_range = bpm >= 30 and bpm <= 220

         if valid_measurements and human_range:
             bpms.append(bpm)
             spo2s.append(spo2)

         if len(bpms) > 12 and len(spo2s) > 12:
             print((np.mean(bpms), np.mean(spo2s)))
             break;

if __name__ == "__main__":
    main(sys.argv[1:])
