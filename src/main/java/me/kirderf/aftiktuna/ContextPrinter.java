package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.entity.Aftik;

public final class ContextPrinter {
	private final GameInstance game;
	
	public ContextPrinter(GameInstance game) {
		this.game = game;
	}
	
	// Print message if the player is controlling the aftik
	public void printf(Aftik aftik, String message, Object... args) {
		if (aftik == game.getAftik())
			game.out().printf(message, args);
	}
	
	// Print message if the aftik controlled by the player is in the room
	public void printf(Room room, String message, Object... args) {
		if (room == game.getAftik().getRoom())
			game.out().printf(message, args);
	}
}