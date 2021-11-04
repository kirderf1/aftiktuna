package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.Ship;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.*;
import java.util.stream.Collectors;
import java.util.stream.Stream;

public final class GameInstance {
	public static final int EXPECTED_LINE_LENGTH = 60;
	public static final Random RANDOM = new Random();
	
	private final ActionHandler actionHandler = new ActionHandler();
	private final PrintWriter out;
	private final BufferedReader in;
	
	private int beatenLocations = 0;
	private Location location;
	private final Ship ship;
	private Aftik aftik;
	private final List<Aftik> crew;
	
	public GameInstance(PrintWriter out, BufferedReader in) {
		this.out = out;
		this.in = in;
		
		crew = new ArrayList<>(List.of(new Aftik("Cerulean"), new Aftik("Mint")));
		aftik = crew.get(0);
		
		ship = new Ship();
		crew.forEach(aftik1 -> ship.getRoom().addObject(aftik1, 0));
	}
	
	public Aftik getAftik() {
		return aftik;
	}
	
	public Stream<GameObject> getGameObjectStream() {
		return Stream.concat(Stream.of(ship.getRoom()), location.getRooms().stream()).flatMap(Room::objectStream);
	}
	
	public PrintWriter out() {
		return out;
	}
	
	public void run(boolean debugLevel) {
		initLocation(debugLevel);
		out.printf("You're playing as the aftik %s.%n", aftik.getName());
		
		while (true) {
			handleCrewDeaths();
			
			if (checkCharacterStatus()) return;
			
			if (checkShipStatus(debugLevel)) return;
			
			getGameObjectStream().flatMap(Entity.CAST.toStream())
							.filter(Entity::isAlive).forEach(Entity::prepare);
			
			printStatus();
			
			handleUserAction();
			
			actionHandler.handleCreatures(this);
			
			out.println();
		}
	}
	
	private void initLocation(boolean debugLevel) {
		if (debugLevel) {
			location = EarlyTestingLocations.createBlockingLocation();
		} else {
			location = Locations.getRandomLocation();
		}
		
		ship.createEntrance(location.getEntryPos());
		
		aftik.remove();
		location.addAtEntry(aftik);
	}
	
	private void handleCrewDeaths() {
		for (Aftik aftik : List.copyOf(crew)) {
			if (aftik.isDead()) {
				if (this.aftik == aftik)
					printStatus();
				out.printf("%s is dead.%n", aftik.getName());
				
				aftik.remove();
				removeFromCrew(aftik);
			}
		}
	}
	
	//Possible calls to this should be followed up by checkCharacterStatus()
	private void removeFromCrew(Aftik aftik) {
		crew.remove(aftik);
		if (this.aftik == aftik)
			this.aftik = null;
	}
	
	private boolean checkCharacterStatus() {
		if (aftik == null) {
			if (crew.isEmpty()) {
				out.println("You lost.");
				return true;
			} else {
				aftik = crew.get(0);
				out.printf("You're now playing as the aftik %s.%n%n", aftik.getName());
			}
		}
		return false;
	}
	
	private boolean checkShipStatus(boolean debugLevel) {
		if (aftik.getRoom() == ship.getRoom() && aftik.removeItem(ObjectTypes.FUEL_CAN)) {
			printStatus();
			beatenLocations++;
			
			if (debugLevel || beatenLocations >= 3) {
				out.println("You got fuel to your ship.");
				out.println("Congratulations, you won!");
				return true;
			} else {
				out.printf("You got fuel to your ship,%n and proceeded to your next location.%n%n");
				
				ship.separateFromLocation();
				aftik.restoreHealth();
				
				initLocation(false);
			}
		}
		return false;
	}
	
	private void handleUserAction() {
		int result = 0;
		do {
			String input;
			try {
				input = in.readLine();
			} catch(IOException e) {
				e.printStackTrace();
				continue;
			}
			
			result = actionHandler.handleInput(this, input);
		} while (result <= 0);
	}
	
	private void printStatus() {
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
				if (builder.length() + label.length() + 3 <= EXPECTED_LINE_LENGTH)
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