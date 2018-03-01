public static string level_name (Distinst.LogLevel level) {
    switch(level) {
    case Distinst.LogLevel.TRACE:
        return "Trace";
    case Distinst.LogLevel.DEBUG:
        return "Debug";
    case Distinst.LogLevel.INFO:
        return "Info";
    case Distinst.LogLevel.WARN:
        return "Warn";
    case Distinst.LogLevel.ERROR:
        return "Error";
    default:
        return "Unknown";
    }
}

public static int main (string[] args) {
    //  Distinst.log((level, message) => {
    //      stderr.printf ("Log: %s %s\r\n", level_name (level), message);
    //  });

    Distinst.Disks disks = Distinst.Disks.probe ();
    foreach (unowned Distinst.Disk disk in disks.list()) {
        uint8[] disk_path = disk.get_device_path();
        uint64 disk_sectors = disk.get_sectors();
        uint64 disk_sector_size = disk.get_sector_size();
        uint64 disk_size = disk_sectors * disk_sector_size;

        stdout.printf(
            "%.*s: %lu * %lu = %lu MB\n",
            disk_path.length,
            (string) disk_path,
            disk_sectors,
            disk_sector_size,
            disk_size/1000000
        );

        foreach (unowned Distinst.Partition partition in disk.list_partitions()) {
            uint8[] part_path = partition.get_device_path();
            uint64 part_start = partition.get_start_sector();
            uint64 part_end = partition.get_end_sector() + 1;
            uint64 part_sectors = part_end - part_start;
            uint64 part_size = part_sectors * disk_sector_size;
            Distinst.PartitionUsage usage = partition.sectors_used(disk_sector_size);

            stdout.printf("  %.*s:\n", part_path.length, (string) part_path);
            stdout.printf("    Start: %lu\n", (ulong) part_start);
            stdout.printf("    End:   %lu\n", (ulong) part_end);
            stdout.printf("    Size:  %lu MB\n", (ulong) part_size / 1000000);
            
            if (usage.tag == 1) {
                stdout.printf(
                    "    Usage: %lu MB\n",
                    (ulong)(usage.value * disk_sector_size / 1000000)
                );
            }

            string? os = partition.probe_os();
            if (os == null) {
                stdout.printf("    OS:    None\n");
            } else {
                stdout.printf("    OS:    Some(%s)\n", os);
            }
        }
    }

    return 0;
}
