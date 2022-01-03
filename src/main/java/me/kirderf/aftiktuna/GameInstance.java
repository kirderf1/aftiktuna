package me.kirderf.aftiktuna;

import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.command.CommandContext;
import me.kirderf.aftiktuna.command.CommandState;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.location.levels.CrewTestingLocations;
import me.kirderf.aftiktuna.location.levels.LocationSelector;
import me.kirderf.aftiktuna.location.levels.Locations;
import me.kirderf.aftiktuna.object.Identifier;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Entity;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;
import me.kirderf.aftiktuna.print.ActionPrinter;
import me.kirderf.aftiktuna.print.HealthTracker;
import me.kirderf.aftiktuna.print.MessageBuffer;
import me.kirderf.aftiktuna.print.StatusPrinter;
import me.kirderf.aftiktuna.util.StreamUtils;

import java.io.BufferedReader;
import java.io.PrintWriter;
import java.util.Optional;
import java.util.Random;
import java.util.function.Supplier;
import java.util.stream.Stream;

/**
 * The central class of the game, which contains the main game loop.
 */
public final class GameInstance {
	public static final Random RANDOM = new Random();
	
	private static final Supplier<Location> DEBUG_LEVEL = CrewTestingLocations::separationTest;
	
	private final PrintWriter out;
	private final InputReader in;
	private final MessageBuffer messageBuffer;
	private final StatusPrinter statusPrinter;
	
	private final HealthTracker healthTracker = new HealthTracker();
	private final CommandState commandState = new CommandState();
	
	private final LocationSelector locations = new LocationSelector();
	
	private int beatenLocations = 0;
	private Location location;
	private final Crew crew;
	
	private final GameView areaView = new AreaView(this);
	private GameView view = areaView;
	
	public GameInstance(PrintWriter out, BufferedReader in, Runnable prepareForInput) {
		this.out = out;
		this.in = new InputReader(in, prepareForInput);
		
		crew = new Crew();
		
		statusPrinter = new StatusPrinter(out, crew);
		messageBuffer = new MessageBuffer();
	}
	
	public Crew getCrew() {
		return crew;
	}
	
	public Stream<GameObject> getGameObjectStream() {
		return Stream.concat(Stream.of(crew.getShip().getRoom()), location.getAreas().stream()).flatMap(Area::objectStream);
	}
	
	public Optional<Area> lookupArea(Identifier<Area> id) {
		return location.getAreas().stream().filter(area -> area.getId().equals(id)).findAny();
	}
	
	public void run(boolean debugLevel) {
		out.println("Welcome to aftiktuna!");
		out.printf("You're playing as the aftik %s.%n", crew.getAftik().getName());
		
		while (true) {
			if (location == null)
				initLocation(debugLevel);
			
			getGameObjectStream().flatMap(Entity.CAST.toStream())
					.filter(Entity::isAlive).forEach(Entity::prepare);
			
			printPage(false);
			
			ActionPrinter actionOut = new ActionPrinter(messageBuffer, crew);
			
			handleUserAction(actionOut);
			for (Entity entity : StreamUtils.sortedWithRandomTiebreaker(
					getGameObjectStream().flatMap(Entity.CAST.toStream()),
					Entity.TURN_ORDER_COMPARATOR).toList()) {
				if (entity.isAlive() && entity != getCrew().getAftik()) {
					entity.performAction(actionOut);
				}
			}
			healthTracker.tick(crew.getAftik(), actionOut);
			
			handleCrewDeaths();
			
			if (crew.isEmpty()) {
				messageBuffer.flush(out);
				out.println("You lost.");
				return;
			}
			crew.replaceLostControlCharacter(actionOut);
			
			handleShipStatus();
			
			if (noMoreLevels(debugLevel)) {
				messageBuffer.flush(out);
				out.println("Congratulations, you won!");
				return;
			}
		}
	}
	
	public StatusPrinter getStatusPrinter() {
		return statusPrinter;
	}
	
	public void restoreView() {
		view = areaView;
	}
	
	public void setStoreView(Shopkeeper shopkeeper) {
		view = new StoreView(statusPrinter, shopkeeper);
	}
	
	private void initLocation(boolean debugLevel) {
		if (debugLevel) {
			Locations.checkLocations();	//Check for errors in locations
			
			location = DEBUG_LEVEL.get();
		} else {
			location = locations.getRandomLocation(out, in);
		}
		
		crew.getShip().createEntrance(location.getEntryPos());
		
		crew.placeCrewAtLocation(location);
		
		messageBuffer.print("The ship arrives at a new location, and the crew exit the ship.");
	}
	
	private void handleCrewDeaths() {
		
		if (crew.getAftik().isDead()) {
			restoreView();
			printPage(true);
			sleep();
		}
		
		for (Aftik aftik : crew.getCrewMembers()) {
			if (aftik.isDead()) {
				messageBuffer.print("%s is dead.", aftik.getName());
				
				aftik.dropItems();
				aftik.remove();
				crew.removeCrewMember(aftik);
			}
		}
	}
	
	private void handleShipStatus() {
		Ship ship = crew.getShip();
		if (ship.getAndClearIsLaunching()) {
			beatenLocations++;
			
			messageBuffer.print("The ship leaves for the next planet.");
			printPage(false);
			sleep();
			
			ship.separateFromLocation();
			for (Aftik aftik : crew.getCrewMembers()) {
				if (aftik.getArea() != ship.getRoom())
					crew.removeCrewMember(aftik);
				else
					aftik.restoreStatus();
			}
			
			location = null;
		}
	}
	
	private boolean noMoreLevels(boolean debugLevel) {
		return beatenLocations >= (debugLevel ? 1 : 3);
	}
	
	private void handleUserAction(ActionPrinter actionOut) {
		Aftik aftik = crew.getAftik();
		if (aftik.getMind().overridesPlayerInput()) {
			sleep();
			aftik.performAction(actionOut);
		} else {
			commandState.inputPrepare(aftik);
			int result;
			do {
				String input = in.readLine();
				
				CommandContext context = new CommandContext(this, commandState, actionOut);
				try {
					result = view.handleInput(input, context);
				} catch(CommandSyntaxException e) {
					result = context.printNoAction("Unexpected input \"%s\"", input);
				}
				
				if (result <= 0) {
					if (context.shouldShowView())
						printPage(false);
					else
						messageBuffer.flush(out);
				} else {
					var optionalAction = context.getAction();
					optionalAction.ifPresentOrElse(action -> action.accept(actionOut),
							() -> System.out.printf("[Warning] Inconsistency in command handling for input \"%s\". Got action result without action.%n", input));
				}
			} while (result <= 0);
		}
	}
	
	private void printPage(boolean fullStatus) {
		out.println("-".repeat(Main.EXPECTED_LINE_LENGTH));
		view.printView(out);
		out.println();
		messageBuffer.flush(out);
		statusPrinter.printStatus(fullStatus);
	}
	
	private void sleep() {
		try {
			Thread.sleep(2000);
		} catch(InterruptedException ignored) {}
	}
}