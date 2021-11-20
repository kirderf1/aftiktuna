package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.action.InputActionContext;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.location.levels.CrewTestingLocations;
import me.kirderf.aftiktuna.location.levels.Locations;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Entity;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;
import me.kirderf.aftiktuna.print.ActionPrinter;
import me.kirderf.aftiktuna.print.StatusPrinter;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.Random;
import java.util.function.Supplier;
import java.util.stream.Stream;

public final class GameInstance {
	public static final Random RANDOM = new Random();
	
	private static final Supplier<Location> DEBUG_LEVEL = CrewTestingLocations::recruitment;
	
	private final PrintWriter out;
	private final BufferedReader in;
	private final Runnable prepareForInput;
	private final ActionPrinter actionOut;
	private final StatusPrinter statusPrinter;
	
	private final ActionHandler actionHandler = new ActionHandler();
	private final Locations locations = new Locations();
	
	private int beatenLocations = 0;
	private Location location;
	private final Crew crew;
	
	public GameInstance(PrintWriter out, BufferedReader in, Runnable prepareForInput) {
		this.out = out;
		this.in = in;
		this.prepareForInput = prepareForInput;
		
		crew = new Crew();
		
		statusPrinter = new StatusPrinter(out, crew);
		actionOut = new ActionPrinter(crew);
	}
	
	public Crew getCrew() {
		return crew;
	}
	
	public Stream<GameObject> getGameObjectStream() {
		return Stream.concat(Stream.of(crew.getShip().getRoom()), location.getAreas().stream()).flatMap(Area::objectStream);
	}
	
	public void run(boolean debugLevel) {
		initLocation(debugLevel);
		actionOut.print("You're playing as the aftik %s.", crew.getAftik().getName());
		
		while (true) {
			
			if (noMoreLevels(debugLevel)) {
				actionOut.flush(out);
				out.println("Congratulations, you won!");
				return;
			}
			
			printStatusAndMessages(false);
			
			getGameObjectStream().flatMap(Entity.CAST.toStream())
							.filter(Entity::isAlive).forEach(Entity::prepare);
			
			handleUserAction();
			actionHandler.handleEntities(this, actionOut);
			
			printSeparatorLine();
			
			handleCrewDeaths();
			handleShipStatus(debugLevel);
			
			if (crew.isEmpty()) {
				actionOut.flush(out);
				out.println("You lost.");
				return;
			}
			
			crew.replaceLostControlCharacter(actionOut);
		}
	}
	
	public void setControllingAftik(Aftik aftik) {
		crew.setControllingAftik(aftik, actionOut);
		
		printSeparatorLine();
		printStatusAndMessages(true);
	}
	
	public StatusPrinter getStatusPrinter() {
		return statusPrinter;
	}
	
	public void runTrade(Aftik aftik, Shopkeeper shopkeeper) {
		out.printf("%s starts trading with the shopkeeper.%n", aftik.getName());
		int result = 0;
		do {
			String input;
			try {
				prepareForInput.run();
				input = in.readLine();
			} catch(IOException e) {
				e.printStackTrace();
				continue;
			}
			
			result = StoreCommands.handleInput(input, new StoreCommands.StoreContext(aftik, shopkeeper, out));
		} while (result <= 0);
	}
	
	private void initLocation(boolean debugLevel) {
		if (debugLevel) {
			Locations.checkLocations();	//Check for errors in locations
			
			location = DEBUG_LEVEL.get();
		} else {
			location = locations.getRandomLocation();
		}
		
		crew.getShip().createEntrance(location.getEntryPos());
		
		crew.placeCrewAtLocation(location);
	}
	
	private void handleCrewDeaths() {
		
		if (crew.getAftik().isDead()) {
			printStatusAndMessages(true);
			sleep();
			printSeparatorLine();
		}
		
		for (Aftik aftik : crew.getCrewMembers()) {
			if (aftik.isDead()) {
				actionOut.print("%s is dead.", aftik.getName());
				
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
				actionOut.print("The ship moves on to the next location.");
				
				ship.separateFromLocation();
				for (Aftik aftik : crew.getCrewMembers()) {
					if (aftik.getArea() != ship.getRoom())
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
			sleep();
			aftik.performAction(actionOut);
		} else {
			int result = 0;
			do {
				String input;
				try {
					prepareForInput.run();
					input = in.readLine();
				} catch(IOException e) {
					e.printStackTrace();
					continue;
				}
				
				result = actionHandler.handleInput(new InputActionContext(this, out, actionOut), input);
			} while (result <= 0);
		}
	}
	
	private void printSeparatorLine() {
		out.println("-".repeat(Main.EXPECTED_LINE_LENGTH));
	}
	
	private void printStatusAndMessages(boolean fullStatus) {
		statusPrinter.printArea(crew.getAftik().getArea());
		out.println();
		actionOut.flush(out);
		statusPrinter.printStatus(fullStatus);
	}
	
	private void sleep() {
		try {
			Thread.sleep(2000);
		} catch(InterruptedException ignored) {}
	}
}