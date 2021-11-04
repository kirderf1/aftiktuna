package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.entity.Aftik;

import java.io.PrintWriter;
import java.util.*;
import java.util.stream.Collectors;

public final class StatusPrinter {
	private final PrintWriter out;
	
	StatusPrinter(PrintWriter out) {
		this.out = out;
	}
	
	void printStatus(Aftik aftik) {
		printRoom(aftik.getRoom());
		printHealth(aftik);
		optionallyPrintWieldedItem(aftik);
		optionallyPrintInventory(aftik);
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
			char symbol = object.getType().symbol();
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
	
	private void printHealth(Aftik aftik) {
		StringBuilder builder = new StringBuilder();
		for (int i = 0; i < 5; i++) {
			builder.append(i * aftik.getMaxHealth() < 5 * aftik.getHealth() ? '#' : '.');
		}
		out.printf("Health: %s%n", builder);
	}
	
	private void optionallyPrintWieldedItem(Aftik aftik) {
		aftik.getWieldedItem().ifPresent(wielded ->
				out.printf("Wielded: %s%n", wielded.capitalizedName()));
	}
	
	private void optionallyPrintInventory(Aftik aftik) {
		List<ObjectType> inventory = aftik.getInventory();
		if (!inventory.isEmpty()) {
			String itemList = inventory.stream().map(ObjectType::capitalizedName).collect(Collectors.joining(", "));
			out.printf("Inventory: %s%n", itemList);
		}
	}
}