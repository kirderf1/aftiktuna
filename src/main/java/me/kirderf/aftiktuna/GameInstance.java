package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.Ship;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.Random;
import java.util.stream.Stream;

public final class GameInstance {
	public static final Random RANDOM = new Random();
	
	private final StatusPrinter statusPrinter;
	private final PrintWriter out;
	private final ContextPrinter contextPrinter;
	private final BufferedReader in;
	
	private final ActionHandler actionHandler = new ActionHandler();
	private final Locations locations = new Locations();
	
	private int beatenLocations = 0;
	private Location location;
	private final Crew crew;
	
	public GameInstance(PrintWriter out, BufferedReader in) {
		this.out = out;
		this.in = in;
		statusPrinter = new StatusPrinter(out);
		
		crew = new Crew();
		contextPrinter = new ContextPrinter(out, crew);
	}
	
	public Aftik getAftik() {
		return getCrew().getAftik();
	}
	
	public Crew getCrew() {
		return crew;
	}
	
	public Stream<GameObject> getGameObjectStream() {
		return Stream.concat(Stream.of(crew.getShip().getRoom()), location.getRooms().stream()).flatMap(Room::objectStream);
	}
	
	public ContextPrinter out() {
		return contextPrinter;
	}
	
	public PrintWriter directOut() {
		return out;
	}
	
	public void run(boolean debugLevel) {
		initLocation(debugLevel);
		out.printf("You're playing as the aftik %s.%n", crew.getAftik().getName());
		
		while (true) {
			handleCrewDeaths();
			handleShipStatus(debugLevel);
			
			if (crew.isEmpty()) {
				out.println("You lost.");
				return;
			}
			
			crew.replaceLostControlCharacter(out);
			
			if (noMoreLevels(debugLevel)) {
				out.println("Congratulations, you won!");
				return;
			}
			
			getGameObjectStream().flatMap(Entity.CAST.toStream())
							.filter(Entity::isAlive).forEach(Entity::prepare);
			
			printStatus();
			
			handleUserAction();
			
			actionHandler.handleEntities(this);
			
			out.println();
		}
	}
	
	public void setControllingAftik(Aftik aftik) {
		crew.setControllingAftik(aftik, out);
	}
	
	public void printStatus() {
		statusPrinter.printStatus(crew.getAftik());
	}
	
	private void initLocation(boolean debugLevel) {
		if (debugLevel) {
			location = CrewTestingLocations.separationTest();
		} else {
			location = locations.getRandomLocation();
		}
		
		crew.getShip().createEntrance(location.getEntryPos());
		
		crew.placeCrewAtLocation(location);
	}
	
	private void handleCrewDeaths() {
		for (Aftik aftik : crew.getCrewMembers()) {
			if (aftik.isDead()) {
				if (crew.getAftik() == aftik)
					printStatus();
				out.printf("%s is dead.%n", aftik.getName());
				
				aftik.dropItems();
				aftik.remove();
				crew.removeCrewMember(aftik);
			}
		}
	}
	
	private void handleShipStatus(boolean debugLevel) {
		Ship ship = crew.getShip();
		if (ship.getAndClearIsLaunching()) {
			beatenLocations++;
			
			if (!noMoreLevels(debugLevel)) {
				out.printf("The ship moves on to the next location.%n");
				
				ship.separateFromLocation();
				for (Aftik aftik : crew.getCrewMembers()) {
					if (aftik.getRoom() != ship.getRoom())
						crew.removeCrewMember(aftik);
					else
						aftik.restoreStatus();
				}
				
				initLocation(false);
			}
		}
	}
	
	private boolean noMoreLevels(boolean debugLevel) {
		return beatenLocations >= (debugLevel ? 1 : 3);
	}
	
	private void handleUserAction() {
		Aftik aftik = crew.getAftik();
		if (aftik.getMind().overridesPlayerInput()) {
			aftik.performAction(contextPrinter);
		} else {
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
}