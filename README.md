## TODOs

* Handle Unicode
  * Currently, we're assuming all input is ASCII
* Currently, we parse instructions every time they're executed
  * We could parse all instructions at object-file load time, potentially speeding up execution