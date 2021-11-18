package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.AftikNPC;
import me.kirderf.aftiktuna.object.entity.Stats;

import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public final class Crew {
	private static final int MAX_SIZE = 2;
	
	private final Ship ship;
	private final List<Aftik> crewMembers;
	private Aftik aftik;
	
	Crew() {
		ship = new Ship();
		crewMembers = new ArrayList<>(List.of(new Aftik("Cerulean", new Stats(9, 2, 10), this),
				new Aftik("Mint", new Stats(10, 3, 8), this)));
		aftik = crewMembers.get(0);
		crewMembers.forEach(aftik1 -> ship.getRoom().addObject(aftik1, 0));
	}
	
	public Aftik getAftik() {
		return aftik;
	}
	
	public Ship getShip() {
		return ship;
	}
	
	/**
	 * Returns a copy of the crew members list (meaning that it is safe to call removeCrewMember() while iterating over this list)
	 */
	public List<Aftik> getCrewMembers() {
		return List.copyOf(crewMembers);
	}
	
	public Optional<Aftik> findByName(String name) {
		for (Aftik aftik : crewMembers) {
			if (aftik.getName().equalsIgnoreCase(name))
				return Optional.of(aftik);
		}
		return Optional.empty();
	}
	
	public boolean isEmpty() {
		return crewMembers.isEmpty();
	}
	
	public boolean hasCapacity() {
		return crewMembers.size() < MAX_SIZE;
	}
	
	void setControllingAftik(Aftik aftik, PrintWriter out) {
		if (!crewMembers.contains(aftik))
			throw new IllegalArgumentException("Aftik must be part of the crew.");
		this.aftik = aftik;
		out.printf("You're now playing as the aftik %s.%n%n", aftik.getName());
	}
	
	void placeCrewAtLocation(Location location) {
		for (Aftik aftik : crewMembers) {
			aftik.remove();
			location.addAtEntry(aftik);
		}
	}
	
	void addCrewMember(AftikNPC npc, PrintWriter out) {
		if (hasCapacity()) {
			Aftik aftik = npc.createAftikForCrew(this);
			crewMembers.add(aftik);
			
			out.printf("%s joined the crew!%n", aftik.getName());
		}
	}
	
	void removeCrewMember(Aftik aftik) {
		crewMembers.remove(aftik);
	}
	
	void replaceLostControlCharacter(PrintWriter out) {
		if (!crewMembers.contains(aftik)) {
			setControllingAftik(crewMembers.get(0), out);
		}
	}
}