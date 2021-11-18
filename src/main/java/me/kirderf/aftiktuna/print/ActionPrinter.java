package me.kirderf.aftiktuna.print;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.entity.Entity;

import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.List;

public final class ActionPrinter {
	private final Crew crew;
	private final List<String> messages = new ArrayList<>();
	
	public ActionPrinter(Crew crew) {
		this.crew = crew;
	}
	
	public void print(String message, Object... args) {
		messages.add(message.formatted(args));
	}
	
	// Print message if the player is controlling the aftik
	public void printFor(Entity entity, String message, Object... args) {
		if (entity == crew.getAftik())
			print(message, args);
	}
	
	// Print message if the aftik controlled by the player is in the area
	public void printAt(Area area, String message, Object... args) {
		if (area == crew.getAftik().getArea())
			print(message, args);
	}
	
	public void printAt(GameObject object, String message, Object... args) {
		printAt(object.getArea(), message, args);
	}
	
	public void flush(PrintWriter out) {
		for (String message : messages) {
			out.println(message);
		}
		messages.clear();
	}
}