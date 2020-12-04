# General idea
The general idea is to create a massively multiplayer game with focus on player interaction and building and programming computers.

## Player interaction
The player interaction should be one not much unlike normal human interaction, by that i don't mean your day to day interactions with
neighbors, family or co-workers. But more something resembling a state-less, order-less and most importantly society-less interaction.
Not much unlike the interaction seens i games like Rust and DayZ. First and foremost the it's a game about survival of the fittest,
and the game design should encourage this, by some external force requireing either stealing or working together with other players.
Although the game should be about survival of the fittest, the hope is that large clans or maybe even societies with different jobs 
may appear.

## Player goals
While player interaction is the what should drive the game, the goals and ways of optaining these goals, are after all the basis for
any game. The goal of the game should be ataining power and resources. This is done at first manually, and thereafter by your computer
driven workforce. The reward is therefore the satisfaction of growth not unlike factorio.

# Inspiration

Tech computer part:
 * Shenzhen
 * Exapunks

Playerinteraction part:
 * Rust (the game)
 * DayZ

 Other:
 * Factorio

# Specific design

## Progression
The progression should be both in terms of power and exponential growth resourcegathering, but also in complexity. The more resources
you posess, the more advanced microcontrollers and computerparts you can manifacture, which will allow more suffisticated behaviour.


## Frame
A frame is anything that has a circuit board, examples of this would be a drone. A drone on it's own is no more than a brick, but when
a player hooks certain components up to the the drones peripherals it can all of a sudden fly. If equiped with a transmitter reciever
module, it can connect to a gps server and know it's relative position. If really sophisticated it could connect to a central control
keeping track of a map over the surrounding area, and a list of drones that then survey and collect resources.
Here is a list of frames:
 * Drone
 * Turret
 * Server (A general purpose frame without any inherit peripherals)


## Circuit board
The circuit board is where components are placed and hooked up with the peripherals of the frame. This is done with wires. There are two
types of wires:
 * Power
 * Data (1, 2, 4, 8, 16 bit)
Any component with allways have at least one collection to either type of wire.


## Components
Components are a what drives the behaviour of the frame, all they can do is process data, and don't do anything on their own.
List of components:
 * Logic Gate (And, or, xor, nor, nand, not)
 * Reciever & transmitter
 * Memory
 * Relay
 * Programmable controller
The precise funtion of these will be explained in a more detailed document.


## Resources
In general there should be a set of basic resources, that can only be optained by extracting them from the world.
These would be:
 * Iron
 * Copper
 * Silicon
 * Oil

These basic resources can then be further refined and manifactured into a vast set of components.

Optaining the required equipment for this should be difficult, and take a long amount of time. The focus of the game should be on
finding clever solutions to interesting problems, player interaction like trading, stealing and hacking. Not as much basebuilding.
The reason this exists is to encourage the formation of clans. To create a power imbalance between different groups of players.
