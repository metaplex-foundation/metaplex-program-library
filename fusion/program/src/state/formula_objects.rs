use anchor_lang::prelude::*;
use crate::{Item, Ingredient};

#[account]
pub struct Formula {
    // Vector of <Ingredient> objects required to satisy the formula
    // Each <Ingredient> item is 33 bytes
    pub ingredients: Vec<Ingredient>,
    // Vector of <Item> objects to be minted on craft
    pub output_items: Vec<Item>,
}