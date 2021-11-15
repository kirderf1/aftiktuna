package me.kirderf.aftiktuna.location;

public record Position(Area area, int coord) {
	public Position {
		area.verifyValidPosition(coord);
	}
	
	public Position atCoord(int newCoord) {
		return new Position(area, newCoord);
	}
	
	public boolean isAdjacent(Position other) {
		return other.area == this.area && Math.abs(other.coord - this.coord) <= 1;
	}
	
	public Position getPosTowards(int otherCoord) {
		if (this.coord < otherCoord) {
			return atCoord(this.coord + 1);
		} else if (this.coord > otherCoord) {
			return atCoord(this.coord - 1);
		} else return this;
	}
}