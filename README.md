# Frequatuner

## Embedded Rust Equalizer and Tuner


An embedded Rust project that has one of the following two processing modes with audio input:
* Convert audio signal to frequencies with magnitudes, and display them in an animated graphical equalizer
* Detect the pitch of the audio signal and display a tuner (goal note, adjacent notes and distance to goal note, basically)

The processing mode can be switched by pressing a button.


---


Hardware used currently is:
+ ESP32S3-c1, version 1.0, including some of its on-board peripherals
+ an AliExpress ledmatrix of 8x32 Ws2812 LEDS that takes a serpentining string of GRB values as input
+ an AliExpress ADC I2S conversion module that includes 3 line-in options.

Audio processing is done using crates fundsp, pitch_detector, pitch_detection and rustfft.
