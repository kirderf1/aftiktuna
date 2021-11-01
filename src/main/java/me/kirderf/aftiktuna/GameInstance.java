package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;

public final class GameInstance {
	private final ActionHandler actionHandler = new ActionHandler();
	private final Location location;
	private final BufferedReader in;
	private final Aftik aftik;
	
	public GameInstance() {
		location = EarlyTestingLocations.createDeathLocation();
		location.addAtEntry(aftik = new Aftik());
		in = new BufferedReader(new InputStreamReader(System.in));
	}
	
	public void run() {
		while (true) {
			System.out.println();
			
			aftik.getRoom().printRoom();
			printHealth(aftik);
			aftik.optionallyPrintWieldedItem();
			aftik.optionallyPrintInventory();
			
			if (aftik.isDead()) {
				System.out.println("You lost.");
				return;
			}
			
			if (aftik.hasItem(ObjectType.FUEL_CAN)) {
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
				
				result = actionHandler.handleInput(this, input);
			} while (result <= 0);
			
			actionHandler.handleCreatures(this);
		}
	}
	
	public Aftik getAftik() {
		return aftik;
	}
	
	private static void printHealth(Aftik aftik) {
		StringBuilder builder = new StringBuilder();
		for (int i = 0; i < 5; i++) {
			builder.append(i < aftik.getHealth() ? '#' : '.');
		}
		System.out.printf("Health: %s%n", builder);
	}
}