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
import me.kirderf.aftiktuna.object.entity.AftikNPC;
import me.kirderf.aftiktuna.object.entity.Entity;
import me.kirderf.aftiktuna.print.ContextPrinter;
import me.kirderf.aftiktuna.print.StatusPrinter;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.Random;
import java.util.stream.Stream;

public final class GameInstance {
	public static final Random RANDOM = new Random();
	
	private final PrintWriter out;
	private final BufferedReader in;
	private final Runnable prepareForInput;
	private final ContextPrinter actionOut;
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
		actionOut = new ContextPrinter(crew);
	}
	
	public Crew getCrew() {
		return crew;
	}
	
	public Stream<GameObject> getGameObjectStream() {
		return Stream.concat(Stream.of(crew.getShip().getRoom()), location.getAreas().stream()).flatMap(Area::objectStream);
	}
	
	public void run(boolean debugLevel) {
		initLocation(debugLevel);
		out.printf("You're playing as the aftik %s.%n%n", crew.getAftik().getName());
		
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
			
			statusPrinter.printStatus(false);
			
			handleUserAction();
			
			actionHandler.handleEntities(this, actionOut);
			
			actionOut.flush(out);
			
			out.println();
		}
	}
	
	public void setControllingAftik(Aftik aftik) {
		crew.setControllingAftik(aftik, out);
	}
	
	public void recruitAftik(AftikNPC npc) {
		crew.addCrewMember(npc, out);
	}
	
	public StatusPrinter getStatusPrinter() {
		return statusPrinter;
	}
	
	private void initLocation(boolean debugLevel) {
		if (debugLevel) {
			Locations.checkLocations();	//Check for errors in locations
			
			location = CrewTestingLocations.recruitment();
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
					statusPrinter.printStatus(true);
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
			try {
				Thread.sleep(2000);
			} catch(InterruptedException ignored) {}
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
}