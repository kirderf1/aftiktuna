package me.kirderf.aftiktuna.print;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;
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
	
	// Print message if the aftik controlled by the player is in the area
	public void printAt(Area area, String message, Object... args) {
		if (area == crew.getAftik().getArea())
			out.printf(message, args);
	}
	
	public void printAt(GameObject object, String message, Object... args) {
		printAt(object.getArea(), message, args);
	}
}