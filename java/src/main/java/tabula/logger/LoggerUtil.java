package tabula.logger;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.logging.FileHandler;
import java.util.logging.Logger;
import java.util.logging.SimpleFormatter;

public class LoggerUtil {
    private static Logger logger;
    private static FileHandler fileHandler;

    private static final String DATE_FORMAT = "yyyy/MM/dd HH:mm:ss";

    public static void initialize(String logFileName) {
        try {
            Files.createDirectories(Paths.get(logFileName).getParent());
            fileHandler = new FileHandler(logFileName, true);
            fileHandler.setFormatter(new SimpleFormatter());

            logger = Logger.getLogger("logger");

            logger.addHandler(fileHandler);
        } catch (IOException e) {
            System.out.printf("Error with %s, %s", logFileName, e.getMessage());
            System.exit(1);
        }
    }

    public static void debug(String message) {
        var timestamp = new SimpleDateFormat(DATE_FORMAT).format(new Date());
        logger.info(String.format("[DEBUG] [%s] %s", timestamp, message));
        close();
    }

    public static void debugf(String format, Object... args) {
        var finalFormat = "[DEBUG] [%s] " + format;
        var timestamp = new SimpleDateFormat(DATE_FORMAT).format(new Date());
        logger.info(String.format(finalFormat, timestamp, args));
        close();
    }

    public static void error(String message) {
        var timestamp = new SimpleDateFormat(DATE_FORMAT).format(new Date());
        logger.severe(String.format("[ERROR] [%s] %s", timestamp, message));
        close();
    }

    public static void errorf(String format, Object... args) {
        var finalFormat = "[ERROR] [%s] " + format;
        var timestamp = new SimpleDateFormat(DATE_FORMAT).format(new Date());
        logger.severe(String.format(finalFormat, timestamp, args));
        close();
    }

    private static void close() {
        if (fileHandler != null) {
            fileHandler.close();
        }
    }
}

