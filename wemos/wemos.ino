#include <ESP8266WiFi.h>
#include <ESP8266HTTPClient.h>
#include <WiFiClient.h>

#include <Wire.h>
#include "MAX30105.h"
#include "heartRate.h"

#define TMP A0
#define BUZZER D5

MAX30105 particleSensor;

#define SSID "BENINCORE"
#define PASSWORD "3206731555"
#define ENDPOINT "http://54c0-2800-e2-e80-9b4-2135-40e6-ec48-7522.ngrok.io/variables"

void setup()
{
    Serial.begin(115200); // Serial port at 115200 kb/s

    pinMode(TMP, INPUT); // Temperature analog pin

    pinMode(BUZZER, OUTPUT); // Buzzer pin
    digitalWrite(BUZZER, LOW);

    // Initialize sensor
    if (!particleSensor.begin(Wire, I2C_SPEED_FAST)) //Use default I2C port, 400kHz speed
    {
        Serial.println("MAX30105 was not found. Please check wiring/power.");
        while (1);
    }

    particleSensor.setup(); //Configure sensor with default settings
    particleSensor.setPulseAmplitudeRed(0x0A); //Turn Red LED to low to indicate sensor is running
    particleSensor.setPulseAmplitudeGreen(0); //Turn off Green LED

    WiFi.begin(SSID, PASSWORD);
    Serial.println("Connecting...");
    while (WiFi.status() != WL_CONNECTED)
    {
        delay(500);
        Serial.print(".");
    }
}

void loop()
{
    // Check finger presence
    long irValue = particleSensor.getIR();

    if (irValue < 5000)
    {
        Serial.println(" No finger?");
    }
    else
    {
        beep();
        float avgBPM = readBPM(10000); // Read BPM for 10 seconds.

        if (avgBPM < 0)
        {
            Serial.println(" No finger?");
        }
        else
        {
            Serial.print("average-BPM: ");
            Serial.println(avgBPM);

            float avgTmp = readTemperature(10000); // Read temperature for 10 seconds.

            Serial.print("average-temperature: ");
            Serial.println(avgTmp);

            postReq(avgBPM, avgTmp);
            buzz(avgBPM, avgTmp, 5000); // Sound for 5 seconds
        }

        beep();
    }

    delay(10000);
}

float readBPM(unsigned int span)
{
    long lastBeat = 0; //Time at which the last beat occurred
    float beatsPerMinute;
    long lastIRvalue;
    float avgBeat = -1;

    unsigned long start = millis();
    unsigned long current = start;

    while (current - start <= span)
    {
        long irValue = particleSensor.getIR();
        lastIRvalue = irValue;

        if (checkForBeat(irValue) == true)
        {
            long delta = millis() - lastBeat;
            lastBeat = millis();

            beatsPerMinute = 60 / (delta / 1000.0);

            if (beatsPerMinute > 20  && beatsPerMinute < 255)
            {
                if (avgBeat < 0)
                {
                    avgBeat = beatsPerMinute;;
                }
                else
                {
                    avgBeat = (avgBeat + beatsPerMinute) / 2;
                }
            }
        }

        current = millis();
    }

    return avgBeat;
}

const float WEMOS_VOLTAGE = 3.3;

// Read temperature for a given span of time and return
// average temperature.
float readTemperature(unsigned int span)
{
    float registerTmp;
    float vout;
    float temperature;
    float avgTemperature = -1;

    unsigned long start = millis();
    unsigned long current = start;

    while (current - start <= span)
    {
        registerTmp = analogRead(TMP); // Read temperature
        vout = (WEMOS_VOLTAGE * registerTmp) / 1023;

        temperature = vout / 0.01;

        if (avgTemperature < 0)
        {
            avgTemperature = temperature;
        }
        else
        {
            avgTemperature = (avgTemperature + temperature) / 2;
        }

        current = millis();
        delay(250); // Delay between readings for performance purposes
    }

    return avgTemperature;
}

const float MIN_BPM = 60;
const float MAX_BPM = 100;
//const float MAX_TMP = 37.5;
const float MAX_TMP = 30.5;

const unsigned int MAX_FREQ = 1100;

void buzz(float bpm, float tmp, unsigned int span)
{
    byte abnormalBpm = bpm < MIN_BPM || bpm > MAX_BPM;
    byte highTmp = tmp > MAX_TMP;
    byte normalIndicators = !abnormalBpm && !highTmp;

    unsigned long start = millis();
    unsigned long current = start;

    unsigned int  freq = MAX_FREQ;

    while (current - start <= span)
    {
        if (normalIndicators)
        {
            digitalWrite(BUZZER, LOW);
            break;
        }

        if (abnormalBpm)
        {
            digitalWrite(BUZZER, HIGH);
            freq -= 300;
        }

        if (highTmp)
        {
            digitalWrite(BUZZER, HIGH);
            freq -= 300;
        }

        current = millis();
        delay(freq);
        digitalWrite(BUZZER, LOW);
        delay(freq);
        freq = MAX_FREQ;
    }
}

void beep()
{
    digitalWrite(BUZZER, HIGH);
    delay(250);
    digitalWrite(BUZZER, LOW);
}

void postReq(float bpm, float temperature)
{
    WiFiClient client;
    HTTPClient http;

    http.begin(client, ENDPOINT);
    http.addHeader("Content-Type", "application/json");

    String bpmTxt = String(bpm, 3);
    String tmpTxt = String(temperature, 3);

    String payload = "{\"bpm\":" + bpmTxt + ",\"temperature\":" + tmpTxt + "}";
    int httpResponseCode = http.POST(payload);

    Serial.print("HTTP Response code: ");
    Serial.println(httpResponseCode);
    http.end();
}
