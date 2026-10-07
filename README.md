# Crabby
- Iced remake of a fun childhood game with a bit more automation in gameplay

### Storytime
- Basically I was playing an old not-to-be-named bumping game on max difficulty and I was sure the AI was cheating with its dice rolls, like even when I was playing completely optimally with 3 humans against it, it would get BS rolls all the time
- Also just wanted to stop using compatability layers to be able to play the game

### Todo
- [ ] Finish the roll logic (pinches, reverses, swaps, move splits)
- [ ] Finish the board logic (slicks, turbos)
- [x] Add a basic AI that just chooses the first option
- [ ] Add an intermediate AI that calculates the total distance left for all of its pieces for each option and chooses the min distance for itself
- [ ] Add an expert AI that also factors in oponents positions/total distance remaining

### Demo
![Demo](demo.png)

### Moves
1) Move a car from your pit row to your start space, or move a car on the track one space forward
2) Move a car on the track two spaces forward
3) Move a car on the track three spaces forward and then take another move ticket
4) Move a car on the track four spaces forward
5) Move a car on the track five spaces forward
6) Move a car on the track six spaces forward or nine spaces backward
7) Move a car on the track seven spaces forward, or switch one of your cars with another player's car (if possible**)
8) Move a car on the track eight spaces forward or split the move between two cars
9) Move a car on the track nine spaces forward or six spaces backward
10) Move a car on the track ten spaces forward
11) Move a car on the track eleven spaces forward, or one space backward
12) Move a car from your pit row to your start space, or move a car on the track twelve spaces forward
13) [Pinch] Move a car from your pit row onto a space with another player's car, and send that car back to pit row!
'em!