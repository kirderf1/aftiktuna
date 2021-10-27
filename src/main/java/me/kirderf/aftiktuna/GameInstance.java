package me.kirderf.aftiktuna;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.OptionalInt;

public class GameInstance {
	private final Room room;
	private final BufferedReader in;
	private final GameObject aftik, fuelCan;
	
	public GameInstance() {
		room = new Room(5);
		room.addObject(aftik = new GameObject('A', "Aftik"), 1);
		room.addObject(fuelCan = new GameObject('f', "Fuel can"), 4);
		in = new BufferedReader(new InputStreamReader(System.in));
	}
	
	public void run() {
		boolean winCondition = false;
		while (true) {
			room.printRoom();
			
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
					OptionalInt pos = room.getPosition(fuelCan);
					if (pos.isPresent()) {
						room.moveObject(aftik, pos.getAsInt());
						room.removeObject(fuelCan);
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
