#!/bin/bash

until solana account $1 2>/dev/null
do
   echo waiting account $1
done