package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Room;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;

import java.io.PrintWriter;
import java.util.*;
import java.util.stream.Collectors;

public final class StatusPrinter {
	private final PrintWriter out;
	private final Crew crew;
	
	private ObjectType shownWielded = ObjectTypes.SHIP_EXIT;	//Dummy value so that the status is printed the first time the print function is called
	private List<ObjectType> shownInventory = List.of(ObjectTypes.SHIP_EXIT);
	private float shownHealth = -1;
	
	StatusPrinter(PrintWriter out, Crew crew) {
		this.out = out;
		this.crew = crew;
	}
	
	void printStatus(boolean fullStatus) {
		Aftik aftik = crew.getAftik();
		printRoom(aftik.getRoom());
		
		printHealth(aftik, fullStatus);
		printWieldedItem(aftik, fullStatus);
		printInventory(aftik, fullStatus);
	}
	
	private void printRoom(Room room) {
		
		Map<GameObject, Character> symbolTable = new HashMap<>();
		Map<Character, String> nameTable = new HashMap<>();
		
		buildTables(room, symbolTable, nameTable);
		
		printRoomMap(room, symbolTable);
		printRoomLabels(nameTable);
	}
	
	private void buildTables(Room room, Map<GameObject, Character> symbolTable, Map<Character, String> nameTable) {
		
		char spareSymbol = '0';
		for (GameObject object : room.objectStream()
				.sorted(Comparator.comparing(GameObject::hasCustomName, Boolean::compareTo))	//Let objects without a custom name get chars first
				.collect(Collectors.toList())) {
			char symbol = object.getDisplaySymbol();
			String name = object.getDisplayName(false, true);
			if (nameTable.containsKey(symbol) && !name.equals(nameTable.get(symbol)))
				symbol = spareSymbol++;
			
			symbolTable.put(object, symbol);
			nameTable.put(symbol, name);
		}
	}
	
	private void printRoomMap(Room room, Map<GameObject, Character> symbolTable) {
		List<List<GameObject>> objectsByPos = new ArrayList<>();
		for (int pos = 0; pos < room.getLength(); pos++)
			objectsByPos.add(new ArrayList<>());
		
		room.objectStream().forEach(object -> objectsByPos.get(object.getCoord()).add(object));
		
		for (List<GameObject> objectStack : objectsByPos)
			objectStack.sort(Comparator.comparingInt(GameObject::getWeight).reversed());
		
		int lines = Math.max(1, objectsByPos.stream().map(List::size).max(Integer::compare).orElse(0));
		
		for (int line = lines - 1; line >= 0; line--) {
			StringBuilder builder = new StringBuilder((line == 0 ? "_" : " ").repeat(room.getLength()));
			for (int pos = 0; pos < room.getLength(); pos++) {
				if (objectsByPos.get(pos).size() > line)
					builder.setCharAt(pos, symbolTable.get(objectsByPos.get(pos).get(line)));
			}
			out.println(builder);
		}
	}
	
	private void printRoomLabels(Map<Character, String> nameTable) {
		StringBuilder builder = new StringBuilder();
		nameTable.forEach((symbol, name) -> {
			String label = "%s: %s".formatted(symbol, name);
			if (!builder.isEmpty()) {
				if (builder.length() + label.length() + 3 <= Main.EXPECTED_LINE_LENGTH)
					builder.append("   ");
				else {
					out.println(builder);
					builder.setLength(0);
				}
			}
			builder.append(label);
		});
		if (!builder.isEmpty())
			out.println(builder);
	}
	
	private void printHealth(Aftik aftik, boolean forcePrint) {
		float health = aftik.getHealth();
		if (forcePrint || shownHealth != health) {
			final int barLength = 10;
			
			StringBuilder builder = new StringBuilder();
			for (int i = 0; i < barLength; i++) {
				builder.append(i * aftik.getMaxHealth() < barLength * health ? '#' : '.');
			}
			out.printf("Health: %s%n", builder);
			shownHealth = health;
		}
	}
	
	private void printWieldedItem(Aftik aftik, boolean forcePrint) {
		aftik.getWieldedItem().ifPresentOrElse(wielded -> {
			if (forcePrint || shownWielded != wielded)
				out.printf("Wielded: %s%n", wielded.capitalizedName());
			shownWielded = wielded;
		}, () -> {
			if (forcePrint || shownWielded != null)
				out.printf("Wielded: Nothing%n");
			shownWielded = null;
		});
	}
	
	private void printInventory(Aftik aftik, boolean forcePrint) {
		List<ObjectType> inventory = aftik.getInventory();
		if (forcePrint || !shownInventory.equals(inventory)) {
			if (!inventory.isEmpty()) {
				String itemList = inventory.stream().map(ObjectType::capitalizedName).collect(Collectors.joining(", "));
				out.printf("Inventory: %s%n", itemList);
			} else {
				out.printf("Inventory: Empty%n");
			}
			shownInventory = List.copyOf(inventory);
		}
	}
}