package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.location.Location;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.function.Supplier;

public final class LocationCategory {
	private final String name;
	private final List<Supplier<Location>> locations;
	
	public LocationCategory(String name, List<Supplier<Location>> locations) {
		this.name = name;
		this.locations = locations;
	}
	
	public String getName() {
		return name;
	}
	
	public LocationCategory mutableCopy() {
		return new LocationCategory(name, new ArrayList<>(locations));
	}
	
	public boolean isEmpty() {
		return locations.isEmpty();
	}
	
	public Location createAndRemoveRandom(Random random) {
		return locations.remove(random.nextInt(locations.size())).get();
	}
	
	public void checkLocations() {
		locations.forEach(Supplier::get);
	}
}