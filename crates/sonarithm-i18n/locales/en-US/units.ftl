# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0

### Units (keyboard-and-navigation §8.1, DL-P12). Every unit message takes a number
### $value and a $style: "symbol" for visual text, braille and the log; "terse" and
### "verbose" for speech. Unit symbols stay in visual text; speech uses the names.
### sonarithm-i18n's unit table (src/units.rs) maps UCUM codes and symbols to these IDs.

unit-ampere =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } ampere
               *[other] { $value } amperes
            }
        [terse]
            { $value ->
                [one] { $value } amp
               *[other] { $value } amps
            }
       *[symbol] { $value } A
    }

unit-celsius =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } degree Celsius
               *[other] { $value } degrees Celsius
            }
        [terse]
            { $value ->
                [one] { $value } degree C
               *[other] { $value } degrees C
            }
       *[symbol] { $value } °C
    }

unit-centimeter =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } centimeter
               *[other] { $value } centimeters
            }
        [terse]
            { $value ->
                [one] { $value } centimeter
               *[other] { $value } centimeters
            }
       *[symbol] { $value } cm
    }

unit-day =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } day
               *[other] { $value } days
            }
        [terse]
            { $value ->
                [one] { $value } day
               *[other] { $value } days
            }
       *[symbol] { $value } d
    }

unit-decibel =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } decibel
               *[other] { $value } decibels
            }
        [terse]
            { $value ->
                [one] { $value } decibel
               *[other] { $value } decibels
            }
       *[symbol] { $value } dB
    }

unit-fahrenheit =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } degree Fahrenheit
               *[other] { $value } degrees Fahrenheit
            }
        [terse]
            { $value ->
                [one] { $value } degree F
               *[other] { $value } degrees F
            }
       *[symbol] { $value } °F
    }

unit-gram =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } gram
               *[other] { $value } grams
            }
        [terse]
            { $value ->
                [one] { $value } gram
               *[other] { $value } grams
            }
       *[symbol] { $value } g
    }

unit-hectopascal =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } hectopascal
               *[other] { $value } hectopascals
            }
        [terse]
            { $value ->
                [one] { $value } hectopascal
               *[other] { $value } hectopascals
            }
       *[symbol] { $value } hPa
    }

unit-hertz =
    { $style ->
        [verbose] { $value } hertz
        [terse] { $value } hertz
       *[symbol] { $value } Hz
    }

unit-hour =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } hour
               *[other] { $value } hours
            }
        [terse]
            { $value ->
                [one] { $value } hour
               *[other] { $value } hours
            }
       *[symbol] { $value } h
    }

unit-kelvin =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } kelvin
               *[other] { $value } kelvins
            }
        [terse]
            { $value ->
                [one] { $value } kelvin
               *[other] { $value } kelvins
            }
       *[symbol] { $value } K
    }

unit-kilogram =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } kilogram
               *[other] { $value } kilograms
            }
        [terse]
            { $value ->
                [one] { $value } kilogram
               *[other] { $value } kilograms
            }
       *[symbol] { $value } kg
    }

unit-kilohertz =
    { $style ->
        [verbose] { $value } kilohertz
        [terse] { $value } kilohertz
       *[symbol] { $value } kHz
    }

unit-kilometer =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } kilometer
               *[other] { $value } kilometers
            }
        [terse]
            { $value ->
                [one] { $value } kilometer
               *[other] { $value } kilometers
            }
       *[symbol] { $value } km
    }

unit-kilometer-per-hour =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } kilometer per hour
               *[other] { $value } kilometers per hour
            }
        [terse]
            { $value ->
                [one] { $value } kilometer per hour
               *[other] { $value } kilometers per hour
            }
       *[symbol] { $value } km/h
    }

unit-kilowatt =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } kilowatt
               *[other] { $value } kilowatts
            }
        [terse]
            { $value ->
                [one] { $value } kilowatt
               *[other] { $value } kilowatts
            }
       *[symbol] { $value } kW
    }

unit-meter =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } meter
               *[other] { $value } meters
            }
        [terse]
            { $value ->
                [one] { $value } meter
               *[other] { $value } meters
            }
       *[symbol] { $value } m
    }

unit-meter-per-second =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } meter per second
               *[other] { $value } meters per second
            }
        [terse]
            { $value ->
                [one] { $value } meter per second
               *[other] { $value } meters per second
            }
       *[symbol] { $value } m/s
    }

unit-millimeter =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } millimeter
               *[other] { $value } millimeters
            }
        [terse]
            { $value ->
                [one] { $value } millimeter
               *[other] { $value } millimeters
            }
       *[symbol] { $value } mm
    }

unit-minute =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } minute
               *[other] { $value } minutes
            }
        [terse]
            { $value ->
                [one] { $value } minute
               *[other] { $value } minutes
            }
       *[symbol] { $value } min
    }

unit-pascal =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } pascal
               *[other] { $value } pascals
            }
        [terse]
            { $value ->
                [one] { $value } pascal
               *[other] { $value } pascals
            }
       *[symbol] { $value } Pa
    }

unit-part-per-million =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } part per million
               *[other] { $value } parts per million
            }
        [terse]
            { $value ->
                [one] { $value } part per million
               *[other] { $value } parts per million
            }
       *[symbol] { $value } ppm
    }

unit-percent =
    { $style ->
        [verbose] { $value } percent
        [terse] { $value } percent
       *[symbol] { $value }%
    }

unit-second =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } second
               *[other] { $value } seconds
            }
        [terse]
            { $value ->
                [one] { $value } second
               *[other] { $value } seconds
            }
       *[symbol] { $value } s
    }

unit-volt =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } volt
               *[other] { $value } volts
            }
        [terse]
            { $value ->
                [one] { $value } volt
               *[other] { $value } volts
            }
       *[symbol] { $value } V
    }

unit-watt =
    { $style ->
        [verbose]
            { $value ->
                [one] { $value } watt
               *[other] { $value } watts
            }
        [terse]
            { $value ->
                [one] { $value } watt
               *[other] { $value } watts
            }
       *[symbol] { $value } W
    }

# A unit the table does not know, printed as the data declares it.
unit-other = { $value } { $unit }
