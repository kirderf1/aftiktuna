package me.kirderf.aftiktuna;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.Optional;
import java.util.OptionalInt;

public class GameInstance {
	private final Location location;
	private final BufferedReader in;
	private final GameObject aftik;
	
	public GameInstance() {
		location = EarlyTestingLocations.createLocation3();
		location.room.addObject(aftik = new GameObject('A', "Aftik", 10), location.entryPoint);
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
					OptionalInt aftikPos = location.room.getPosition(aftik);
					
					Optional<Room.PlacedObject> optionalFuel = location.room.findNearest(GameObject::isFuelCan, aftikPos.orElseThrow());
					if (optionalFuel.isPresent()) {
						
						location.room.moveObject(aftik, optionalFuel.get().pos());
						location.room.removeObject(optionalFuel.get().gameObj());
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
