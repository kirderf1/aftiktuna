package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;

public final class GameInstance {
	private final Location location;
	private final BufferedReader in;
	private final Aftik aftik;
	
	public GameInstance() {
		location = EarlyTestingLocations.createDoorLocation3();
		location.addAtEntry(aftik = new Aftik());
		in = new BufferedReader(new InputStreamReader(System.in));
	}
	
	public void run() {
		while (true) {
			System.out.println();
			aftik.getRoom().printRoom();
			aftik.optionallyPrintInventory();
			
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
				
				result = ActionHandler.handleInput(this, input);
			} while (result <= 0);
		}
	}
	
	public Aftik getAftik() {
		return aftik;
	}
}