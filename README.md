# LC-3 in Rust

This is a reasonably complete implementation of the [LC-3b](https://en.wikipedia.org/wiki/Little_Computer_3), an architectural specification for an educational 16-bit computer. The ISA is described [here](http://users.ece.utexas.edu/~patt/07s.360N/handouts/360n.appC.pdf) (or [here](https://web.archive.org/web/20230826043728/http://users.ece.utexas.edu/~patt/07s.360N/handouts/360n.appC.pdf) if the link is dead).


## TODOs

* Handle Unicode
  * Currently, we're assuming all input is ASCII
* Currently, we parse instructions every time they're executed
  * We could parse all instructions at object-file load time, potentially speeding up execution


## Links

* Hex editors
  * https://codepen.io/isdampe/pen/YQoVaK
  * https://codepen.io/masik/pen/ACpGl


## License

GPLv2, see LICENSE file.
