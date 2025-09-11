#include <IRremote.h>

#define RECV_PIN        2       // Receiver signal pin

#define MOTOR_PWM       480     // Motor power [0-799]
#define MOTOR_REVERSE   false   // Reverse motor direction
#define PWM_MAX_VALUE   799     // 799 for 16kHz. (don't touch if you don't know how)

IRrecv irrecv(RECV_PIN);
decode_results results;

void setup() {
  // setup PWM frequency for pins 9 and 10:
  TCCR1A = 0;
  TCCR1B = 0;
  TCCR1A = (1 << COM1A1) | (1 << COM1B1) | (1 << WGM11);
  TCCR1B = (1 << WGM13) | (1 << WGM12) | (1 << CS10);

  ICR1 = PWM_MAX_VALUE; // (16MHz / (1 * (799 + 1)) )

  OCR1A = 0;
  OCR1B = 0;

  // init motor pins:
  pinMode(9, OUTPUT);
  pinMode(10, OUTPUT);
  
  Serial.begin(9600);
  irrecv.enableIRIn();
}


long light_code = 0xFF9A65;
bool light_status = false;

// Enable/disable light in room
void light(bool enable) {
  // motor to right:
  if (enable) {
    analogWrite9(LOW);
    analogWrite10(MOTOR_PWM);
  }
  // motor to left:
  else {
    analogWrite10(LOW);
    analogWrite9(MOTOR_PWM);
  }

  // wait some time:
  delay(100);

  // disable motor:
  analogWrite9(LOW);
  analogWrite10(LOW);
}


void loop() {
  // decode receiver code:
  if (irrecv.decode(&results)) {
    unsigned long code = results.value;

    // send to com port:
    Serial.println(code, HEX);

    // turn on/off light:
    if (code == light_code) {
      light_status = !light_status;
      light(light_status);
    }
    
    // continue receiving:
    irrecv.resume();
  }
}
 

// Analog write for 9 pin (16kHz)
void analogWrite9(uint16_t duty) {
  OCR1A = constrain(duty, 0, PWM_MAX_VALUE);
}

// Analog write for 10 pin (16kHz)
void analogWrite10(uint16_t duty) {
  OCR1B = constrain(duty, 0, PWM_MAX_VALUE);
}
