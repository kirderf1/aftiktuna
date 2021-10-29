package me.kirderf.aftiktuna;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.OptionalInt;

public class GameInstance {
	private final Location location;
	private final BufferedReader in;
	private final GameObject aftik;
	
	public GameInstance() {
		location = EarlyTestingLocations.createLocation1();
		location.room.addObject(aftik = new GameObject('A', "Aftik"), location.entryPoint);
		in = new BufferedReader(new InputStreamReader(System.in));
	}
	
	public void run() {
		boolean winCondition = false;
		while (true) {
			location.room.printRoom();
			
			if (winCondition) {
				System.out.println("Congratulations, you won!");
				return;
			}
			
			while(true) {
				String input;
				try {
					input = in.readLine();
				} catch(IOException e) {
					e.printStackTrace();
					continue;
				}
				if (input.equals("take fuel can")) {
					OptionalInt pos = location.room.getPosition(location.fuelCan);
					if (pos.isPresent()) {
						location.room.moveObject(aftik, pos.getAsInt());
						location.room.removeObject(location.fuelCan);
						System.out.println("You picked up the fuel can.");
						
						winCondition = true;
					} else {
						System.out.println("There is no fuel can here to pick up.");
					}
					break;
				} else {
					System.out.printf("Unexpected input \"%s\"%n", input);
				}
			}
		}
	}
}
