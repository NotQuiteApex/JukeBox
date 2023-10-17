// #include <Arduino.h>

// #include "serialhelp.h"

// Stages of serial communication
enum SerialStage
{
  Handshake,      // The initial stage, where the computer and MCU greet
  ComputerParts,  // The computer sends some constant data over the wire
  ContinuousStats // The computer sends data that gets updated over time
} commstage;

// The parts of comm stages (stage recieve and response)
int commstagepart;

// The data string we use to store received data
String inputString = "";

// A temporary string for sorting data in the serial files.
String tempInput = "";

// Data we store!
// Data that we receive only at the time a connection is made.
String cpuName = "-";
String gpuName = "-";
String ramCount = "-";
// Data we constantly receive after a connection is made.
String cpuFreq = "-";
String cpuTemp = "-";
String cpuLoad = "-";
String ramUsed = "-";
String gpuTemp = "-";
String gpuCoreClock = "-";
String gpuCoreLoad = "-";
String gpuVramClock = "-";
String gpuVramLoad = "-";

extern ScreenState screenstate;

extern u32 counter;
extern u32 countermax;

void serial_begin() {
  // Set up serial communication for the board
  #if defined (TEENSYDUINO)
    Serial.begin(115200); // Teensy doesn't have settings for Serial.
  #else
    Serial.begin(115200, SERIAL_8E2);
  #endif

  // Initialize the communications stage
  commstage = SerialStage::Handshake;
  commstagepart = 0;

  // Reserve some space for the strings
  inputString.reserve(125);
  tempInput.reserve(50);

  cpuName.reserve(30);
  gpuName.reserve(40);
  ramCount.reserve(4);

  cpuFreq.reserve(4);
  cpuTemp.reserve(4);
  cpuLoad.reserve(4);
  ramUsed.reserve(4);
  gpuTemp.reserve(4);
  gpuCoreClock.reserve(8);
  gpuCoreLoad.reserve(4);
  gpuVramClock.reserve(8);
  gpuVramLoad.reserve(6);
}

void serial_loop() {
  static byte errorcount = 0;
  // we mustve lost comms, reset!
  if (errorcount >= 5) {
    errorcount = 0;

    commstage = SerialStage::Handshake;
    commstagepart = 0;

    screenstate = ScreenState::WaitingConnection;

    cpuName = "-";
    gpuName = "-";
    ramCount = "-";

    cpuFreq = "-";
    cpuTemp = "-";
    cpuLoad = "-";
    ramUsed = "-";
    gpuTemp = "-";
    gpuCoreClock = "-";
    gpuCoreLoad = "-";
    gpuVramClock = "-";
    gpuVramLoad = "-";
  }

    if (commstage == SerialStage::Handshake) {
      if (commstagepart == 0) {
        if (serial_matches("010")) {
          commstagepart = 1;
        }
      } else if (commstagepart == 1) {
        Serial.write("101");
        commstage = SerialStage::ComputerParts;
        commstagepart = 0;
      }
    } else if (commstage == SerialStage::ComputerParts) {
      if (commstagepart == 0) {
        // wait for info on CPU, GPU, and RAM
        if (receive_once_data()) {
          commstagepart = 1;
          errorcount = 0;
        }
        else {
          errorcount++;
          countermax = 1000; //delay(1000);
        }
      } else if (commstagepart == 1) {
        // respond that we got it, and move to the next stage
        Serial.write("222");
        commstage = SerialStage::ContinuousStats;
        screenstate = ScreenState::ShowStats;
        commstagepart = 0;
      }
    } else if (commstage == SerialStage::ContinuousStats) {
      if (commstagepart == 0) {
        // recieve
        if (receive_cont_data()) {
          commstagepart = 1;
          errorcount = 0;
        } else {
          errorcount++;
          countermax = 1000; //delay(1000);
        }
      } else if (commstagepart == 1) {
        // respond that we got it, and repeat
        Serial.write("333");
        commstagepart = 0;
      }
    }
}

void serial_receive()
{
  inputString = "";
  if (Serial.available() > 0)
  {
    while (Serial.available() > 0)
    {
      // Read in data to a string
      int readbyte = Serial.read();
      if (readbyte != -1)
        inputString += (char)readbyte;
    }
  }
}

bool serial_matches(char * matcher)
{
  serial_receive();

  return inputString.equals(matcher);
}

bool receive_once_data()
{
  // string format of computer parts stats
  // $"{cpuName}|{gpuName}|{ramTotal}GB|"

  // recieve data
  serial_receive();

  // if data isn't empty, process it
  if (!inputString.equals(""))
  {
    // process the names!
    tempInput = "";

    // beginning index and end index, and the variable to store data into
    unsigned int idx1 = 0;
    unsigned int idx2 = 0;
    byte count = 0;

    // Loop through the string, finding the pipe seperators
    for (unsigned int i = 0; i < inputString.length(); i++)
    {
      // if character matches a pipe, we process the data before
      if (inputString[i] == '|')
      {
        // Old start index, to end index
        idx1 = idx2;
        if (inputString[idx1] == '|')
          idx1++;
        idx2 = i;

        // slice string, grab the data
        tempInput = inputString.substring(idx1, idx2);

        // hand off data to proper variable
        if (count == 0)
          cpuName = tempInput;
        else if (count == 1)
          gpuName = tempInput;
        else if (count == 2)
          ramCount = tempInput;

        // update variable to determine what variable to hand off to next
        count++;
      }
    }

    return true; // we sucessfully processed data
  }

  return false; // we didn't have any data to process, we bounce
}

bool receive_cont_data()
{
  // Format string of continuous stats
  // $"{cpuFreq}|{cpuTemp}|{cpuLoad}|{ramUsed}|{gpuTemp}|" +
  // $"{gpuCoreClock}|{gpuCoreLoad}|{gpuVramClock}|{gpuVramLoad}|"

  // this is the same as above, just with more variables to hand data to

  serial_receive();
  if (!inputString.equals(""))
  {
    tempInput = "";

    unsigned int idx1 = 0;
    unsigned int idx2 = 0;
    byte count = 0;

    for (unsigned int i = 0; i < inputString.length(); i++)
    {
      if (inputString[i] == '|')
      {
        idx1 = idx2;
        if (inputString[idx1] == '|')
          idx1++;
        idx2 = i;

        tempInput = inputString.substring(idx1, idx2);

        if (count == 0)
          cpuFreq = tempInput;
        else if (count == 1)
          cpuTemp = tempInput;
        else if (count == 2)
          cpuLoad = tempInput;
        else if (count == 3)
          ramUsed = tempInput;
        else if (count == 4)
          gpuTemp = tempInput;
        else if (count == 5)
          gpuCoreClock = tempInput;
        else if (count == 6)
          gpuCoreLoad = tempInput;
        else if (count == 7)
          gpuVramClock = tempInput;
        else if (count == 8)
          gpuVramLoad = tempInput;

        count++;
      }
    }

    return true;
  }

  return false;
}
