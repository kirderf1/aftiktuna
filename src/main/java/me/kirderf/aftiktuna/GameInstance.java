package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.Ship;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.stream.Stream;

public final class GameInstance {
	public static final Random RANDOM = new Random();
	
	private final ActionHandler actionHandler = new ActionHandler();
	private final StatusPrinter statusPrinter;
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
		statusPrinter = new StatusPrinter(out);
		
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
			handleShipStatus(debugLevel);
			
			if (crew.isEmpty()) {
				out.println("You lost.");
				return;
			}
			replaceLostControlCharacter();
			
			if (noMoreLevels(debugLevel)) {
				out.println("Congratulations, you won!");
				return;
			}
			
			getGameObjectStream().flatMap(Entity.CAST.toStream())
							.filter(Entity::isAlive).forEach(Entity::prepare);
			
			statusPrinter.printStatus(aftik);
			
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
					statusPrinter.printStatus(aftik);
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
	
	//Assumes that the crew isn't empty
	private void replaceLostControlCharacter() {
		if (aftik == null) {
			aftik = crew.get(0);
			out.printf("You're now playing as the aftik %s.%n%n", aftik.getName());
		}
	}
	
	private void handleShipStatus(boolean debugLevel) {
		if (aftik != null && aftik.getRoom() == ship.getRoom() && aftik.removeItem(ObjectTypes.FUEL_CAN)) {
			statusPrinter.printStatus(aftik);
			beatenLocations++;
			
			if (noMoreLevels(debugLevel)) {
				out.println("You got fuel to your ship.");
			} else {
				out.printf("You got fuel to your ship,%n and proceeded to your next location.%n%n");
				
				ship.separateFromLocation();
				for (Aftik aftik : List.copyOf(crew)) {
					if (aftik.getRoom() != ship.getRoom())
						removeFromCrew(aftik);
				}
				
				crew.forEach(Entity::restoreHealth);
				
				initLocation(false);
			}
		}
	}
	
	private boolean noMoreLevels(boolean debugLevel) {
		return beatenLocations >= (debugLevel ? 1 : 3);
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
}