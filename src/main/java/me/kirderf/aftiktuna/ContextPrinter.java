package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.entity.Entity;

public final class ContextPrinter {
	private final GameInstance game;
	
	public ContextPrinter(GameInstance game) {
		this.game = game;
	}
	
	// Print message if the player is controlling the aftik
	public void printFor(Entity entity, String message, Object... args) {
		if (entity == game.getAftik())
			game.out().printf(message, args);
	}
	
	// Print message if the aftik controlled by the player is in the room
	public void printAt(Room room, String message, Object... args) {
		if (room == game.getAftik().getRoom())
			game.out().printf(message, args);
	}
	
	public void printAt(Entity entity, String message, Object... args) {
		printAt(entity.getRoom(), message, args);
	}
}