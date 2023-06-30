# Linear State Machine

This project explores writing state machines in a more linear way and making them type safe.
It achieves this by heavily using Rust type system, ownership and async implementation.
The project was developed as part of my Master's thesis.

Primary use case is to write state machines for embedded devices and robotics, where a 
program is often a state machine that works with resources (ie. sensors, outputs, motors, ...). 
Idea comes from the `async/await` syntax and how it simplifies writing futures. 
This project tries to do the same for state machines.

It does this in three parts:
- Simplifying the state machine with the use of `async/await` syntax.
- Using Rust type system to make working with resources type safe.
- Providing a `lin-state` library that implements guards, which are used to ensure the 
    state of the machine during longer action.
    Example of such use is a program that controls a motor which can only move if the doors
    of the machine are closed. We can use a door guard to ensure that the doors are closed.
    If at any point the doors are opened, the motor will stop. 

## Usage

TODO: Write this chapter.
