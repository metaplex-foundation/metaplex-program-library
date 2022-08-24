## Metaplex Candy Guard

> 🛑 **DO NOT USE IN PRODUCTION**: This repository contain a proof-of-concept.

The new `Candy Guard` program is designed to take away the **access control** logic from the `Candy Machine` to handle the additional mint features, while the `Candy Machine` program retains its core mint functionality – the creation of the NFT. This not only provides a clear separation between **access controls** and **mint logic**, it also provides a modular and flexible architecture to add or remove mint features without having to modify the `Candy Machine` program.

The access control on a `Candy Guard` is encapsulated in individuals guards representing a specific rule that needs to be satisfied, which can be enabled or disabled. For example, the live date of the mint is represented as the `LiveDate` guard. This guard is satisfied only if the transaction time is on or after the configured start time on the guard. Other guards can validate different aspects of the access control – e.g., ensuring that the user holds a specific token (white listing).

### Overview

The main purpose of the `CandyGuard` program is to hold the configuration of mint **guards** and apply them before a user can mint from a candy machine. If all enabled guard conditions are valid, the mint transaction is forwarded to the candy machine.

When a mint transaction is received, the program performs the following steps:

1. Validates the transaction against all enabled guards.
    - If any of the guards fail at this point, the transaction is subject to the `bot tax` (when the `bot tax` guard is enabled).
2. After evaluating that all guards are valid, it invokes the `pre_actions` function on each guard. This function is responsible to perform any action **before** the mint (e.g., take payment for the mint).
3. Then the transaction is forwarded to the `Candy Machine` program to mint the NFT. 
4. Finally, it invokes the `post_actions` function on each enabled guard. This function is responsible to perform any action **after** the mint (e.g., freeze the NFT, change the update authority).

A **guard** is a modular piece of code that can be easily added to the `Candy Guard` program, providing great flexibility and simplicity to support specific features without having to modify directly the `Candy Machine` program. Adding new guards is supported by conforming to specific interfaces, with changes isolated to the individual guard – e.g., each guard can be created and modified in isolation. It also provides the flexibility to enable/disable guards without requiring code changes.

### Guards (provisional)

The `Candy Guard` program contains a set of core guards that can be enabled/disabled:

- **bot tax**: configurable tax (amount) to charge invalid transactions
- **live date**: controls when the mint is allowed
- **lamports**: set the price of the mint in SOL
- **spl-token**: set the price of the mint in spl-token amount
- **whitelist**: allows the creation of a whitelist with specific mint date and discount price
- **end settings**: determines a rule to end the mint period based on time or amount
- **gatekeeper**: captcha integration
