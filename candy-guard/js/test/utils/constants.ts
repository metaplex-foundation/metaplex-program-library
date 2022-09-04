// Bytes offset for the start of the data section:
//     8 (discriminator)
//  + 32 (base)
//  +  1 (bump)
//  + 32 (authority)
export const DATA_OFFSET = 8 + 32 + 1 + 32;
