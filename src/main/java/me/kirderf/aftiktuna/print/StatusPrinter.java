package me.kirderf.aftiktuna.print;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.Main;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Room;
import me.kirderf.aftiktuna.object.entity.Aftik;

import java.io.PrintWriter;
import java.util.*;
import java.util.stream.Collectors;

public final class StatusPrinter {
	private final PrintWriter out;
	private final Crew crew;
	
	//Printer for the aftik that is being controlled
	private AftikPrinter aftikPrinter;
	
	public StatusPrinter(PrintWriter out, Crew crew) {
		this.out = out;
		this.crew = crew;
	}
	
	public void printStatus(boolean fullStatus) {
		Aftik aftik = crew.getAftik();
		printRoom(aftik.getRoom());
		
		if (!(aftikPrinter != null && aftikPrinter.isForAftik(aftik))) {
			aftikPrinter = new AftikPrinter(out, aftik);
			aftikPrinter.printAftik(true);
		} else
			aftikPrinter.printAftik(fullStatus);
	}
	
	public void printCrewStatus() {
		out.printf("Crew:%n");
		for (Aftik aftik : crew.getCrewMembers()) {
			new AftikPrinter(out, aftik).printAftikWithName();
		}
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
}