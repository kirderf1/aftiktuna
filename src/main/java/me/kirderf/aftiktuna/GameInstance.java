package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.List;

public final class GameInstance {
	private final Location location;
	private final BufferedReader in;
	private final GameObject aftik;
	private final List<ObjectType> inventory = new ArrayList<>();
	
	public GameInstance() {
		location = EarlyTestingLocations.createDoorLocation2();
		location.addAtEntry(aftik = new GameObject(ObjectType.AFTIK, 10));
		in = new BufferedReader(new InputStreamReader(System.in));
	}
	
	public void run() {
		while (true) {
			aftik.getRoom().printRoom();
			
			if (inventory.contains(ObjectType.FUEL_CAN)) {
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
	
	public void addItem(ObjectType type) {
		inventory.add(type);
	}
	
	public GameObject getAftik() {
		return aftik;
	}
}