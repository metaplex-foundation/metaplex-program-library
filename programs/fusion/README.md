# Metaplex Fusion

The Metaplex fusion contract is a flexible protocol allowing the creation of complex formulas.

## Instruction Set

There are two instructions, `CreateFormula` and `Craft`.

### CreateFormula

CreateFormula takes in a list of Ingredients and a list of output Items. Ingredients in a formula can take an amount and a boolean to tell whether the ingredient should be burned on Craft.

If outputs are simple SPL Tokens then the mint_authority on the mint is transfered to a PDA. If an output item is marked as a Metaplex MasterEdition, then the MasterEdition master token is transfered to a newly created TokenAccount owned by the program. Lastly, the Formula data is stored on chain.

### Craft

The Craft instruction checks that the signer owns all the minimum amounts of ingredients required. If the Formula specifies the ingredient should be burned on Craft then the user's tokens are burned.

Then we loop through the formula's output items. For each item if it's a simple SPL Token the tokens are minted to the signer. If it is marked as a MasterEdition, then a new print is created using a CPI to the Token-Metadata contract.

### Example Use Cases

#### Gaming

- Alchemy: 3 herbs + 2 cranberries = 1 health potion
  - These items could all be common items, where they are SPL Token Mint's wrapped with Token Metadata but are not prints
-

## Roadmap

- [ ] Allow an ingredient to be a MasterEdition, where any print of that master would meet the requirements.
- [ ] Random attribute selection for printing a generative output
