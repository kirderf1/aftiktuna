package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Room;
import me.kirderf.aftiktuna.object.entity.Entity;

import java.io.PrintWriter;

public final class ContextPrinter {
	private final PrintWriter out;
	private final Crew crew;
	
	public ContextPrinter(PrintWriter out, Crew crew) {
		this.out = out;
		this.crew = crew;
	}
	
	// Print message if the player is controlling the aftik
	public void printFor(Entity entity, String message, Object... args) {
		if (entity == crew.getAftik())
			out.printf(message, args);
	}
	
	// Print message if the aftik controlled by the player is in the room
	public void printAt(Room room, String message, Object... args) {
		if (room == crew.getAftik().getRoom())
			out.printf(message, args);
	}
	
	public void printAt(GameObject object, String message, Object... args) {
		printAt(object.getRoom(), message, args);
	}
}