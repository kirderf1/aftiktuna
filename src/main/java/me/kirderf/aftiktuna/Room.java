package me.kirderf.aftiktuna;

import java.util.ArrayList;
import java.util.List;

public final class Room {
	private final int length;
	private final List<PlacedObject> objects = new ArrayList<>();
	
	public Room(int length) {
		this.length = length;
	}
	
	public void addObject(GameObject object, int position) {
		objects.add(new PlacedObject(object, position));
	}
	
	public void printRoom() {
		StringBuilder builder = new StringBuilder("_".repeat(length));
		for (PlacedObject object : objects)
			builder.setCharAt(object.pos, object.gameObj.getSymbol());
		System.out.println(builder);
		for (PlacedObject object : objects)
			System.out.println(object.gameObj.getSymbol() + ": " + object.gameObj.getName());
	}
	
	private record PlacedObject(GameObject gameObj, int pos) {}
}
