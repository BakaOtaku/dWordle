# DWordle

Overview
Wordle is an online 5-letter word game. Each day a new word is released and players have six attempts to guess what the word of the day is. During the guesses, tiles will change color to help players get the word. A grey letter means it isn’t in today’s word, whilst a yellow letter signals it is in the word but in the wrong position. Then there’s the green letter which means it’s in the word and in the right place.
We plan to make an incentivized wordle with player pools to incentivize the best and the faster players for each round of wordle, the fastest players to guess the words get the maximum award from the pool of players

## Why Dwordle?

We propose a new system for wordle that rewards users with nft for each successful guess,  The words are set in the contract at init, and the contract is deployed on Secret network, which encrypts the storage of the smart contract so no one can figure out the set of words beforehand.
We introduce the WLE token. To play the game the user has to lock in n number of WLE tokens to be eligible to play the game. This WLE token is locked in a smart contract that keeps track of the player for that round of wordle. (each round is of 24 hours/ 1 day)
The system also rewards the users who solve the wordle first from the pool of tokens WL on the basis of how many users solved the wordle after that user.

## How we prevent the system from spam .

There can be an instance where a user can find the right wordle and then can flood the system with correct wordle predictions from a different address thus causing issues for other players.  To discourage this
In a day there are 3 words to be guessed and each user gets a word from these 3 words to be guessed.  Thus discouraging the attacker from flooding the system as each prediction take n wle tokens to be deposited and the next prediction for a new address can or can not be the previous.

## Play to Earn
At end of the round, the users can claim their rewards, and rewards for each user are calculated on the basis of the number of pool participants and who solved the wordle first.
Each successful submission is rewarded with an nft that is the proof that a user has completed the wordle challenge
