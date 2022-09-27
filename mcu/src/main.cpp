#include <Arduino.h>
#include <Keyboard.h>
#include <Bounce2.h>

#define DEBOUNCE_INTERVAL 20

Bounce btns[12] = {};
int pins[] = {
	 2,  3,  4,  5,
	 6,  7,  8,  9,
	15, 14, 16, 10,
};
int k_fn[] = {
	KEY_F13, KEY_F14, KEY_F15, KEY_F16,
	KEY_F17, KEY_F18, KEY_F19, KEY_F20,
	KEY_F21, KEY_F22, KEY_F23, KEY_F24
};

void setup() {
	// put your setup code here, to run once:

	// setup all the debouncing
	for (byte i = 0; i < 12; i++) {
		btns[i] = Bounce();
		btns[i].attach(pins[i], INPUT_PULLUP);
		btns[i].interval(DEBOUNCE_INTERVAL);
	}

	// setup keyboard over usb
	Keyboard.begin();
}

void loop() {
  // put your main code here, to run repeatedly:
  // update all the buttons first
  for (byte i=0; i<12; i++) {
    btns[i].update();
  }

  // check all the buttons states
  for (byte i = 0; i < 12; i++) {
    if (btns[i].changed()) {
      int read = btns[i].read();
      if (read == HIGH) {
        Keyboard.release(k_fn[i]);
      } else {
        Keyboard.press(k_fn[i]);
      }
    }
  }
}