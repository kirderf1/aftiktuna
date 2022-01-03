package me.kirderf.aftiktuna.print;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Entity;

/**
 * Interface for sending action messages. Checks if the message should be seen before passing it over to the MessageBuffer.
 */
public final class ActionPrinter implements SimplePrinter {
	private final MessageBuffer buffer;
	private final Aftik aftik;
	private final Area originalArea;
	
	public ActionPrinter(MessageBuffer buffer, Crew crew) {
		this.buffer = buffer;
		this.aftik = crew.getAftik();
		this.originalArea = aftik.getArea();
	}
	
	@Override
	public void println() {
		buffer.println();
	}
	
	@Override
	public void print(String message, Object... args) {
		buffer.print(message, args);
	}
	
	// Print message if the player is controlling the aftik
	public void printFor(Entity entity, String message, Object... args) {
		if (entity == aftik)
			print(message, args);
	}
	
	// Print message if the aftik controlled by the player is in the area
	public void printAt(Area area, String message, Object... args) {
		if (area == aftik.getArea())
			print(message, args);
	}
	
	public void printAt(GameObject object, String message, Object... args) {
		printAt(object.getArea(), message, args);
	}
	
	// Print message if the aftik was in the area at the start of the tick. Used for movement messages.
	public void printFrom(Area area, String message, Object... args) {
		if (area == originalArea)
			print(message, args);
	}
}