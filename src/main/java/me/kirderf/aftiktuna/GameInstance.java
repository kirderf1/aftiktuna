package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Location;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.Optional;

public final class GameInstance {
	private final Location location;
	private final BufferedReader in;
	private final GameObject aftik;
	
	public GameInstance() {
		location = EarlyTestingLocations.createLocation3();
		location.addAtEntry(aftik = new GameObject('A', "Aftik", 10));
		in = new BufferedReader(new InputStreamReader(System.in));
	}
	
	public void run() {
		boolean winCondition = false;
		while (true) {
			aftik.getRoom().printRoom();
			
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
					Optional<GameObject> optionalFuel = aftik.findNearest(GameObject::isFuelCan);
					if (optionalFuel.isPresent()) {
						
						aftik.move(optionalFuel.get().getPosition());
						optionalFuel.get().remove();
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
