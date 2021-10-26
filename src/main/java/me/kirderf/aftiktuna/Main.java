package me.kirderf.aftiktuna;

public final class Main {
	
	public static void main(String[] args) {
		System.out.println("Hello universe");
		System.out.println();
		
		Room room = new Room(5);
		room.addObject(new GameObject('A', "Aftik"), 1);
		room.addObject(new GameObject('f', "Fuel can"), 4);
		room.printRoom();
	}
}