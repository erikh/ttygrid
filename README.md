# ttygrid: reactive layouts for terminal tables

ttygrid makes your tables into reactive layouts. You feed it your contents and
the columns you want to show, and it will calculate what to show based on the
length of text in the table column, and the "priority", an ascending number
indicating display priority within the table.

The result is something like [this](https://asciinema.org/a/609115) with [the
demo source here](examples/demo.rs). Padding is allocated for all columns so
the tables are presented nicely and orderly.

**ttygrid does not work with stream I/O. Terminal I/O only!**. At this point,
you must detect if you are a TTY _before_ invoking ttygrid calls.

ttygrid uses crossterm underneath the hood to detect the width of the terminal
as well as manage colors when they are desired.

## Usage

[docs.rs has it all](https://docs.rs/ttygrid/).

## Author

Erik Hollensbe <github@hollensbe.org>
