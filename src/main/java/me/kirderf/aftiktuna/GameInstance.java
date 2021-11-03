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
	private Ship ship;
	private final Aftik aftik;
	
	public GameInstance(PrintWriter out, BufferedReader in) {
		this.out = out;
		this.in = in;
		aftik = new Aftik();
	}
	
	private void initLocation(boolean debugLevel) {
		if (debugLevel) {
			location = EarlyTestingLocations.createBlockingLocation();
		} else {
			location = Locations.getRandomLocation();
		}
		
		ship = new Ship();
		ship.createEntrance(location.getEntryPos());
		location.addAtEntry(aftik);
	}
	
	public void run(boolean debugLevel) {
		initLocation(debugLevel);
		
		while (true) {
			getGameObjectStream().flatMap(Entity.CAST.toStream())
							.filter(Entity::isAlive).forEach(Entity::prepare);
			
			printRoom(aftik.getRoom());
			printHealth(aftik);
			optionallyPrintWieldedItem(aftik);
			optionallyPrintInventory(aftik);
			
			if (aftik.isDead()) {
				out.println("You lost.");
				return;
			}
			
			if (aftik.getRoom() == ship.getRoom() && aftik.removeItem(ObjectTypes.FUEL_CAN)) {
				beatenLocations++;
				
				if (debugLevel || beatenLocations >= 3) {
					out.println("You got fuel to your ship.");
					out.println("Congratulations, you won!");
					return;
				} else {
					out.printf("You got fuel to your ship,%n and proceeded to your next location.%n%n");
					
					aftik.remove();
					aftik.restoreHealth();
					initLocation(false);
					continue;
				}
			}
			
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
			
			actionHandler.handleCreatures(this);
			
			out.println();
		}
	}
	
	public Aftik getAftik() {
		return aftik;
	}
	
	public Stream<GameObject> getGameObjectStream() {
		return location.getRooms().stream().flatMap(Room::objectStream);
	}
	
	public PrintWriter out() {
		return out;
	}
	
	private void printRoom(Room room) {
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
					builder.setCharAt(pos, objectsByPos.get(pos).get(line).getType().symbol());
			}
			out.println(builder);
		}
		
		StringBuilder builder = new StringBuilder();
		Set<ObjectType> writtenChars = new HashSet<>();
		room.objectStream().forEach(object -> {
			if (writtenChars.add(object.getType())) {
				String entry = "%s: %s".formatted(object.getType().symbol(), object.getType().name());
				if (!builder.isEmpty()) {
					if(builder.length() + entry.length() + 3 <= EXPECTED_LINE_LENGTH)
						builder.append("   ");
					else {
						out.println(builder);
						builder.setLength(0);
					}
				}
				builder.append(entry);
			}
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
				out.printf("Wielded: %s%n", wielded.name()));
	}
	
	private void optionallyPrintInventory(Aftik aftik) {
		List<ObjectType> inventory = aftik.getInventory();
		if (!inventory.isEmpty()) {
			String itemList = inventory.stream().map(ObjectType::name).collect(Collectors.joining(", "));
			out.printf("Inventory: %s%n", itemList);
		}
	}
}