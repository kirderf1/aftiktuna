package me.kirderf.aftiktuna;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.object.Door;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.Optional;

public final class GameInstance {
	private static final CommandDispatcher<GameInstance> DISPATCHER = new CommandDispatcher<>();
	
	static {
		DISPATCHER.register(literal("take").then(literal("fuel").then(literal("can").executes(context -> context.getSource().takeFuelCan()))));
		DISPATCHER.register(literal("go").then(literal("through").then(literal("door").executes(context -> context.getSource().goThroughDoor()))));
	}
	
	private static LiteralArgumentBuilder<GameInstance> literal(String str) {
		return LiteralArgumentBuilder.literal(str);
	}
	
	private final Location location;
	private final BufferedReader in;
	private final GameObject aftik;
	private boolean winCondition = false;
	
	public GameInstance() {
		location = EarlyTestingLocations.createDoorLocation1();
		location.addAtEntry(aftik = new GameObject('A', "Aftik", 10));
		in = new BufferedReader(new InputStreamReader(System.in));
	}
	
	public void run() {
		while (true) {
			aftik.getRoom().printRoom();
			
			if (winCondition) {
				System.out.println("Congratulations, you won!");
				return;
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
				
				try {
					result = DISPATCHER.execute(input, this);
				} catch(CommandSyntaxException ignored) {
					System.out.printf("Unexpected input \"%s\"%n", input);
				}
			} while (result <= 0);
		}
	}
	
	private int takeFuelCan() {
		Optional<GameObject> optionalFuel = aftik.findNearest(GameObject::isFuelCan);
		if (optionalFuel.isPresent()) {
			
			aftik.move(optionalFuel.get().getPosition());
			optionalFuel.get().remove();
			System.out.println("You picked up the fuel can.");
			
			winCondition = true;
		} else {
			System.out.println("There is no fuel can here to pick up.");
		}
		return 1;
	}
	
	private int goThroughDoor() {
		Optional<Door> optionalDoor = aftik.findNearest(GameObject::getAsDoor);
		if (optionalDoor.isPresent()) {
			
			aftik.move(optionalDoor.get().getDestination());
			System.out.println("You entered the door into a new room.");
		} else {
			System.out.println("There is no door here to go through.");
		}
		return 1;
	}
}