# Interaction Net Runtime in Rust

This project is a Rust implementation of an Interaction Net Runtime. Interaction nets are a graphical model for computation, introduced by Yves Lafont in 1990. They have applications in areas like functional programming, proof theory, and the study of complexity in computation.

## What are Interaction Nets?

Interaction nets are a formalism for representing computation as a network of interconnected agents. Each agent represents a function or a rule, and agents are connected by wires. The computation proceeds by interactions between the agents following a local rewriting process. This process eliminates some agents and replaces them with new agents, based on the interaction rules defined for the system.

Interaction nets have a number of interesting properties:

- They are inherently parallel, allowing for efficient parallel implementations.
- They have a simple and concise syntax, making them easy to reason about.
- They can be used to represent a wide range of computational models, including lambda calculus and linear logic.

## Prior Art

The concept of interaction nets was first introduced by Yves Lafont in his 1990 paper, "Interaction Combinators." Since then, there has been a significant amount of research on interaction nets and their applications in various domains.

Some notable examples of prior work include:

- The Geometry of Interaction by Jean-Yves Girard, which relates interaction nets to linear logic and provides a geometric interpretation of computation.
- The Lamping's Abstract Algorithm, which uses interaction nets to implement optimal reductions in the lambda calculus.

## Importance of Interaction Nets

Interaction nets are important for several reasons:

They provide a general and unifying framework for studying computation, applicable to a wide range of models.
Their inherent parallelism makes them well-suited for modern hardware architectures, enabling efficient parallel implementations.
They offer a clear and simple syntax, which allows for easier reasoning and analysis of computational processes.

## Project Overview

This Rust implementation of an Interaction Net Runtime provides a high-performance, concurrent runtime environment for executing interaction nets. Key features of this project include:

Efficient implementation of the core interaction net data structures and operations.
Support for custom interaction rules and agents.
An API for creating, modifying, and executing interaction nets.
Utilities for visualizing and debugging interaction nets.

## Getting Started

TBD
