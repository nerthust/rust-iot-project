#include <ESP8266HTTPClient.h>
#include <ESP8266WiFi.h>
#include <WiFiClient.h>
#include <Wire.h>
#include "heartRate.h"
#include "MAX30105.h"

// TEMPERATURE sensor PIN.
#define TMP A0

// BUZZER PIN.
#define BUZZER D5

// WIFI Configuration.
#define SSID "NETWORK"
#define PASSWORD "xxxxxxxx"

// Endpoint where data is sent.
#define ENDPOINT "http://54c0-2800-e2-e80-9b4-2135-40e6-ec48-7522.ngrok.io/variables"

// Declare particle sensor.
MAX30105 particleSensor;


void setup()
{
    Serial.begin(115200); // Serial port at 115200 kb/s.

    pinMode(TMP, INPUT);        // Temperature analog pin.
    pinMode(BUZZER, OUTPUT);    // Buzzer pin.
    digitalWrite(BUZZER, LOW);  // Initialize buzzer.

    // Initialize sensor if detected, otherwise print error log.
    if (!particleSensor.begin(Wire, I2C_SPEED_FAST)) // Use default I2C port, 400kHz speed.
    {
        Serial.println("MAX30105 was not found. Please check wiring/power.");
        while (1);
    }

    // Configure sensor with default settings.
    particleSensor.setup();

    // Turn Red LED to low to indicate sensor is running.
    particleSensor.setPulseAmplitudeRed(0x0A);

    // Turn off green LED as our sensor doesn't have it.
    particleSensor.setPulseAmplitudeGreen(0);

    // Initialize WiFi connection.
    WiFi.begin(SSID, PASSWORD);
    Serial.println("Connecting...");

    // If not connected to WiFi retry connection every 500ms.
    while (WiFi.status() != WL_CONNECTED)
    {
        delay(500);
        Serial.print(".");
    }
}

void loop()
{
    // Check finger presence by getting measurement on infra-red led.
    long irValue = particleSensor.getIR();

    // Measurement checks only start if finger is detected.
    if (irValue < 5000)
    {
        Serial.println(" No finger?");
    }
    else
    {
        beep();
        float avgBPM = readBPM(5000); // Read BPM for 5 seconds.

        if (avgBPM < 0)
        {
            Serial.println(" No finger?");
        }
        else
        {
            Serial.print("average-BPM: ");
            Serial.println(avgBPM);

            float avgTmp = readTemperature(5000); // Read temperature for 5 seconds.

            Serial.print("average-temperature: ");
            Serial.println(avgTmp);

            postReq(avgBPM, avgTmp);
            buzz(avgBPM, avgTmp, 5000); // Buzz for 5 seconds
        }

        beep();
    }

    delay(2000);
}

// Read BPM for an specified time span in milliseconds.
float readBPM(unsigned int span)
{
    long lastBeat = 0; // Time at which the last beat occurred.
    long lastIRvalue;  // Infra-red led value.
    float n = 0;       // Number of measurements.
    float sum = 0;     // Accumulated readings.

    // Initialize variables for initialize time and current time.
    unsigned long start = millis();
    unsigned long current = start;

    // Loop until time span has elapsed.
    while (current - start <= span)
    {
        // Get value from infra-red sensor.
        long irValue = particleSensor.getIR();
        lastIRvalue = irValue;

        // Check if there was a heart beat.
        if (checkForBeat(irValue) == true)
        {
            // Get difference between current time and last beat
            // in seconds.
            long delta = (millis() - lastBeat) / 1000;

            // Update last beat measurement.
            lastBeat = millis();

            // BPM = 60sec / delta.
            float beatsPerMinute = 60 / delta;

            // Only consider measurements within a reasonable range: (30, 255).
            if (beatsPerMinute > 30  && beatsPerMinute < 255)
            {
                sum += beatsPerMinute;
                n++;
            }
        }

        // Update current time.
        current = millis();

        delay(100); // Delay between readings for performance purposes.
    }

    // Return accumulated average.
    return (sum / n);
}

// Max voltage that WeMos microcontroller can measure.
const float WEMOS_VOLTAGE = 3.3;

// Read temperature for a given span of time and return average temperature.
float readTemperature(unsigned int span)
{
    float n = 0;   // Number of measurements.
    float sum = 0; // Accumulated readings.

    unsigned long start = millis(); // start time.
    unsigned long current = start;  // current time that updates on each iteration.

    while (current - start <= span)
    {
        // TMP reading from analog pin.
        float registerTmp = analogRead(TMP); // Read temperature

        // To compute the temperature the following formula is used:
        // ((Voltage * reading) / 1023) / 0.01.
        float vout = (WEMOS_VOLTAGE * registerTmp) / 1023;
        float temperature = vout / 0.01;

        // Update accumulated sum and number of measurements.
        sum += temperature;
        n++;
        current = millis();

        delay(100); // Delay between readings for performance purposes.
    }

    return (sum / n);
}

// Normal BPM readings should be between 60 and 100 beats per minute when resting.
const float MIN_BPM = 60;
const float MAX_BPM = 100;

// Normal body temperature should be below 37.5.
const float MAX_TMP = 37.5;

// Given a BPM reading and a TEMPERATURE reading, turn BUZZER on for a specific time span in
// milliseconds only if readings are abnormal.
void buzz(float bpm, float tmp, unsigned int span)
{
    // BPM are abnormal if less than MIN_BPM or greater than MAX_BPM.
    byte abnormalBpm = bpm < MIN_BPM || bpm > MAX_BPM;
    // Body temperature is considered high if greater than MAX_TMP.
    byte highTmp = tmp > MAX_TMP;

    // Indicators are normal if both BPM and TEMPERATURE are both ok.
    byte normalIndicators = !abnormalBpm && !highTmp;

    // Delay between buzzer beeps.
    unsigned int freq = 1000;

    unsigned long start = millis();
    unsigned long current = start;

    if (!normalIndicators) {
        // If abnormal BPM reduce beep frequency so that BUZZER buzzes faster.
        if (abnormalBpm)
        {
            freq -= 250;
        }

        // If high TEMPERATURE reduce beep frequency so that BUZZER buzzes faster.
        if (highTmp)
        {
            freq -= 250;
        }


        // Loop for a specific time span.
        while (current - start <= span)
        {
            digitalWrite(BUZZER, HIGH);
            delay(freq);
            digitalWrite(BUZZER, LOW);

            current = millis();
        }

        // Reset frequency.
        freq = 1000;
    }
}

// BEEP for 250 milliseconds.
void beep()
{
    digitalWrite(BUZZER, HIGH);
    delay(250);
    digitalWrite(BUZZER, LOW);
}

// Given BPM and TEMPERATURE readings, post JSON payload to server.
void postReq(float bpm, float temperature)
{
    WiFiClient client;
    HTTPClient http;

    // Initialize HTTP client.
    http.begin(client, ENDPOINT);

    // Add JSON headers.
    http.addHeader("Content-Type", "application/json");

    // Turn readings into Strings.
    String bpmTxt = String(bpm, 3);
    String tmpTxt = String(temperature, 3);

    // Build JSON payload.
    String payload = "{\"bpm\":" + bpmTxt + ",\"temperature\":" + tmpTxt + "}";

    // Perform POST request.
    int httpResponseCode = http.POST(payload);

    // Log HTTP response code.
    Serial.print("HTTP Response code: ");
    Serial.println(httpResponseCode);

    // Finalize HTTP client.
    http.end();
}
