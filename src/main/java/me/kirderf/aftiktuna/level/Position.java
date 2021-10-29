package me.kirderf.aftiktuna.level;

public record Position(Room room, int coord) {
	public Position {
		room.verifyValidPosition(coord);
	}
	
	public Position atCoord(int newCoord) {
		return new Position(room, newCoord);
	}
}