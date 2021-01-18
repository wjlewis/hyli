# HYLI

## Proof-of-Concept

This is a small proof-of-concept for `hyli`, written in Racket.
I'll use this sketch as a foundatin for writing the first full version in Rust.

## Overview

The core consists of a single function, `transform`, that converts a generic tree into an HTML tree.
We'll need to pay more attention to output formatting for the real deal; for instance, right now we're outputting the entire HTML document on a single line, but we probably want to aim for something easier to read.
