package me.kirderf.aftiktuna.level;

public record Position(Room room, int coord) {
	public Position {
		room.verifyValidPosition(coord);
	}
	
	public Position atCoord(int newCoord) {
		return new Position(room, newCoord);
	}
	
	public Position getPosTowards(int otherCoord) {
		if (this.coord < otherCoord) {
			return atCoord(this.coord + 1);
		} else if (this.coord > otherCoord) {
			return atCoord(this.coord - 1);
		} else return this;
	}
}