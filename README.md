# Linear State Machine

This project explores writing programs which are inherently state machines in a more linear way and making them type safe.
It achieves this by heavily using Rust type system, ownership and async implementation.

Specifically, it focuses on easing the writing of programs for embedded devices and robotics,
where a program is often a state machine that works with resources (i.e. sensors, outputs, motors, ...).
Idea comes from the Rust's async implementation, where a future is a state machine and `async/await` syntax
is a syntactic sugar for working with futures. A great explanation of this can be found in
[tokio's tutorial](https://tokio.rs/tokio/tutorial/async). This project aims to do a similar 
simplification for programs that are inherently state machines.

It does this in three parts:
- Simplifying the implementation of state machines with the use of `async/await` syntax.
- Using Rust type system to make working with resources safe. Suppose we have a machine with
    a door and a motor. We only want to be able to move the motor
    if the door is closed. We can use Rust's ownership to ensure that moving the motor
    without first checking the door is closed is a compile-time error.
- Providing a `lin-state` library that implements guards, which are used to ensure the 
    state of the resources is valid during longer action.
    Example of such use is again a program that controls a motor which can only move if the doors
    of the machine are closed. We can use a door guard to ensure that the doors are closed while moving
    the motor. If at any point the doors are opened, the motor will stop moving. In fact, the whole
    block of code inside the guard will stop executing. 

The first two parts are just guidelines on how to write the code. The third part is the library
contained in this repository.

## TO DO

This project is not yet complete. The following is a list of things that still need to be done:
- [ ] Document how to use this project
- [ ] Add examples
- [ ] Document the code
- [ ] Document the design decisions
