// @generated automatically by Diesel CLI.

diesel::table! {
    alarm (id) {
        id -> Int4,
        #[max_length = 30]
        dev_eui -> Varchar,
        min_treshold -> Nullable<Float8>,
        max_treshold -> Nullable<Float8>,
        sms -> Nullable<Bool>,
        email -> Nullable<Bool>,
        temperature -> Nullable<Bool>,
        humadity -> Nullable<Bool>,
        ec -> Nullable<Bool>,
        door -> Nullable<Bool>,
        w_leak -> Nullable<Bool>,
        is_time_limit_active -> Nullable<Bool>,
        alarm_start_time -> Nullable<Float8>,
        alarm_stop_time -> Nullable<Float8>,
        zone_category -> Nullable<Int4>,
        notification -> Nullable<Bool>,
        is_active -> Nullable<Bool>,
        pressure -> Nullable<Bool>,
        #[max_length = 50]
        notification_sound -> Nullable<Varchar>,
        user_id -> Nullable<Array<Nullable<Int8>>>,
        distance -> Nullable<Bool>,
        defrost_time -> Nullable<Int4>,
    }
}

diesel::table! {
    alarm_audit_log (id) {
        id -> Int4,
        alarm_id -> Int4,
        #[max_length = 30]
        dev_eui -> Nullable<Varchar>,
        #[max_length = 10]
        change_type -> Nullable<Varchar>,
        changed_at -> Nullable<Timestamp>,
        changed_by -> Nullable<Int4>,
        old_values -> Nullable<Jsonb>,
        new_values -> Nullable<Jsonb>,
    }
}

diesel::table! {
    alarm_automation_rules (id) {
        alarm_id -> Int4,
        #[max_length = 50]
        receiver_sensor -> Varchar,
        #[max_length = 255]
        action -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        is_active -> Nullable<Bool>,
        user_id -> Int4,
        receiver_device_type -> Nullable<Int4>,
        receiver_device_name -> Nullable<Text>,
        id -> Int4,
    }
}

diesel::table! {
    alarm_date_time (id) {
        alarm_id -> Int4,
        alarm_day -> Int4,
        start_time -> Float8,
        end_time -> Float8,
        id -> Int4,
    }
}

diesel::table! {
    am103 (id) {
        id -> Int4,
        #[max_length = 20]
        dev_eui -> Varchar,
        device_type_id -> Int4,
        org_id -> Int4,
        submission_date -> Nullable<Timestamp>,
        air_temperature -> Nullable<Numeric>,
        air_humidity -> Nullable<Numeric>,
        co2_ppm -> Nullable<Numeric>,
        batv -> Nullable<Int4>,
    }
}

diesel::table! {
    api_key (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        #[max_length = 100]
        name -> Varchar,
        is_admin -> Bool,
        tenant_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    application (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 100]
        name -> Varchar,
        description -> Text,
        mqtt_tls_cert -> Nullable<Bytea>,
        tags -> Jsonb,
    }
}

diesel::table! {
    application_integration (application_id, kind) {
        application_id -> Uuid,
        #[max_length = 20]
        kind -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        configuration -> Jsonb,
    }
}

diesel::table! {
    automation_rules (id) {
        id -> Int4,
        #[max_length = 50]
        sender_sensor -> Nullable<Varchar>,
        #[max_length = 50]
        receiver_sensor -> Nullable<Varchar>,
        #[max_length = 255]
        condition -> Nullable<Varchar>,
        #[max_length = 255]
        action -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        is_active -> Nullable<Bool>,
        user_id -> Int4,
        sender_device_type -> Nullable<Int4>,
        receiver_device_type -> Nullable<Int4>,
        sender_device_name -> Nullable<Text>,
        receiver_device_name -> Nullable<Text>,
        #[max_length = 50]
        trigger_type -> Nullable<Varchar>,
        trigger_time -> Nullable<Timestamp>,
        organization_id -> Nullable<Int4>,
    }
}

diesel::table! {
    dds45lb (id) {
        id -> Int4,
        #[max_length = 45]
        dev_eui -> Varchar,
        device_type_id -> Int4,
        org_id -> Int4,
        submission_date -> Nullable<Timestamp>,
        distance -> Nullable<Int4>,
        batv -> Nullable<Numeric>,
    }
}

diesel::table! {
    device (dev_eui) {
        dev_eui -> Bytea,
        application_id -> Uuid,
        device_profile_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_seen_at -> Nullable<Timestamptz>,
        scheduler_run_after -> Nullable<Timestamptz>,
        #[max_length = 100]
        name -> Varchar,
        description -> Text,
        external_power_source -> Bool,
        battery_level -> Nullable<Numeric>,
        margin -> Nullable<Int4>,
        dr -> Nullable<Int2>,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        altitude -> Nullable<Float4>,
        dev_addr -> Nullable<Bytea>,
        #[max_length = 1]
        enabled_class -> Bpchar,
        skip_fcnt_check -> Bool,
        is_disabled -> Bool,
        tags -> Jsonb,
        variables -> Jsonb,
        join_eui -> Bytea,
        secondary_dev_addr -> Nullable<Bytea>,
        device_session -> Nullable<Bytea>,
        data_time -> Nullable<Int4>,
        organization_id -> Nullable<Int4>,
        temperature_calibration -> Nullable<Numeric>,
        humadity_calibration -> Nullable<Numeric>,
        #[max_length = 100]
        device_profile_name -> Nullable<Varchar>,
        device_type -> Nullable<Int4>,
    }
}

diesel::table! {
    device_data_2025 (id) {
        id -> Int4,
        dev_eui -> Text,
        device_type_id -> Int4,
        air_temperature -> Nullable<Numeric>,
        air_humidity -> Nullable<Numeric>,
        sol_temperature -> Nullable<Numeric>,
        sol_water -> Nullable<Numeric>,
        sol_conduct_soil -> Nullable<Numeric>,
        submission_date -> Nullable<Timestamp>,
        water_leak_status -> Nullable<Int4>,
        water_leak_times -> Nullable<Int4>,
        last_water_leak_duration -> Nullable<Int4>,
        door_open_status -> Nullable<Int4>,
        door_open_times -> Nullable<Int4>,
        last_door_open_duration -> Nullable<Int4>,
        batv -> Nullable<Numeric>,
        ro1_status -> Nullable<Int4>,
        ro2_status -> Nullable<Int4>,
        ph_soil -> Nullable<Numeric>,
        co2_ppm -> Nullable<Numeric>,
        tvoc_ppm -> Nullable<Numeric>,
        sensecap_light -> Nullable<Numeric>,
        barometric_pressure -> Nullable<Numeric>,
        status -> Nullable<Int4>,
        current -> Nullable<Numeric>,
        factor -> Nullable<Numeric>,
        power -> Nullable<Numeric>,
        power_sum -> Nullable<Numeric>,
        voltage -> Nullable<Numeric>,
    }
}

diesel::table! {
    device_data_latest (dev_eui) {
        id -> Int4,
        dev_eui -> Text,
        device_type_id -> Int4,
        org_id -> Int4,
        air_temperature -> Nullable<Numeric>,
        air_humidity -> Nullable<Numeric>,
        sol_temperature -> Nullable<Numeric>,
        sol_water -> Nullable<Numeric>,
        sol_conduct_soil -> Nullable<Numeric>,
        submission_date -> Nullable<Timestamp>,
        water_leak_status -> Nullable<Int4>,
        water_leak_times -> Nullable<Int4>,
        last_water_leak_duration -> Nullable<Int4>,
        door_open_status -> Nullable<Int4>,
        door_open_times -> Nullable<Int4>,
        last_door_open_duration -> Nullable<Int4>,
        batv -> Nullable<Numeric>,
        ro1_status -> Nullable<Int4>,
        ro2_status -> Nullable<Int4>,
        ph_soil -> Nullable<Numeric>,
        co2_ppm -> Nullable<Numeric>,
        tvoc_ppm -> Nullable<Numeric>,
        sensecap_light -> Nullable<Numeric>,
        barometric_pressure -> Nullable<Numeric>,
        current -> Nullable<Numeric>,
        factor -> Nullable<Numeric>,
        power -> Nullable<Numeric>,
        voltage -> Nullable<Numeric>,
        power_sum -> Nullable<Numeric>,
        status -> Nullable<Int4>,
        power_consumption -> Nullable<Int4>,
        switch1 -> Nullable<Int4>,
        switch2 -> Nullable<Int4>,
        switch3 -> Nullable<Int4>,
        switch4 -> Nullable<Int4>,
        switch5 -> Nullable<Int4>,
        switch6 -> Nullable<Int4>,
        switch7 -> Nullable<Int4>,
        switch8 -> Nullable<Int4>,
        #[max_length = 50]
        adc_1 -> Nullable<Varchar>,
        #[max_length = 50]
        adc_2 -> Nullable<Varchar>,
        #[max_length = 50]
        adv_1 -> Nullable<Varchar>,
        #[max_length = 50]
        gpio_in_1 -> Nullable<Varchar>,
        #[max_length = 50]
        gpio_in_2 -> Nullable<Varchar>,
        #[max_length = 50]
        gpio_in_3 -> Nullable<Varchar>,
        #[max_length = 50]
        gpio_in_4 -> Nullable<Varchar>,
        #[max_length = 50]
        gpio_out_1 -> Nullable<Varchar>,
        #[max_length = 50]
        gpio_out_2 -> Nullable<Varchar>,
        distance -> Nullable<Int4>,
        #[max_length = 20]
        position -> Nullable<Varchar>,
        temperature1 -> Nullable<Numeric>,
        temperature2 -> Nullable<Numeric>,
    }
}

diesel::table! {
    device_keys (dev_eui) {
        dev_eui -> Bytea,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        nwk_key -> Bytea,
        app_key -> Bytea,
        dev_nonces -> Jsonb,
        join_nonce -> Int4,
    }
}

diesel::table! {
    device_profile (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 10]
        region -> Varchar,
        #[max_length = 10]
        mac_version -> Varchar,
        #[max_length = 20]
        reg_params_revision -> Varchar,
        #[max_length = 100]
        adr_algorithm_id -> Varchar,
        #[max_length = 20]
        payload_codec_runtime -> Varchar,
        uplink_interval -> Int4,
        device_status_req_interval -> Int4,
        supports_otaa -> Bool,
        supports_class_b -> Bool,
        supports_class_c -> Bool,
        tags -> Jsonb,
        payload_codec_script -> Text,
        flush_queue_on_activate -> Bool,
        description -> Text,
        measurements -> Jsonb,
        auto_detect_measurements -> Bool,
        #[max_length = 100]
        region_config_id -> Nullable<Varchar>,
        allow_roaming -> Bool,
        rx1_delay -> Int2,
        abp_params -> Nullable<Jsonb>,
        class_b_params -> Nullable<Jsonb>,
        class_c_params -> Nullable<Jsonb>,
        relay_params -> Nullable<Jsonb>,
        class_b_timeout -> Nullable<Int4>,
        class_b_ping_slot_nb_k -> Nullable<Int4>,
        class_b_ping_slot_dr -> Nullable<Int2>,
        class_b_ping_slot_freq -> Nullable<Int8>,
        class_c_timeout -> Nullable<Int4>,
        abp_rx1_delay -> Nullable<Int2>,
        abp_rx1_dr_offset -> Nullable<Int2>,
        abp_rx2_dr -> Nullable<Int2>,
        abp_rx2_freq -> Nullable<Int8>,
        is_relay -> Nullable<Bool>,
        is_relay_ed -> Nullable<Bool>,
        relay_ed_relay_only -> Nullable<Bool>,
        relay_enabled -> Nullable<Bool>,
        relay_cad_periodicity -> Nullable<Int2>,
        relay_default_channel_index -> Nullable<Int2>,
        relay_second_channel_freq -> Nullable<Int8>,
        relay_second_channel_dr -> Nullable<Int2>,
        relay_second_channel_ack_offset -> Nullable<Int2>,
        relay_ed_activation_mode -> Nullable<Int2>,
        relay_ed_smart_enable_level -> Nullable<Int2>,
        relay_ed_back_off -> Nullable<Int2>,
        relay_ed_uplink_limit_bucket_size -> Nullable<Int2>,
        relay_ed_uplink_limit_reload_rate -> Nullable<Int2>,
        relay_join_req_limit_reload_rate -> Nullable<Int2>,
        relay_notify_limit_reload_rate -> Nullable<Int2>,
        relay_global_uplink_limit_reload_rate -> Nullable<Int2>,
        relay_overall_limit_reload_rate -> Nullable<Int2>,
        relay_join_req_limit_bucket_size -> Nullable<Int2>,
        relay_notify_limit_bucket_size -> Nullable<Int2>,
        relay_global_uplink_limit_bucket_size -> Nullable<Int2>,
        relay_overall_limit_bucket_size -> Nullable<Int2>,
    }
}

diesel::table! {
    device_profile_template (id) {
        id -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 100]
        name -> Varchar,
        description -> Text,
        #[max_length = 100]
        vendor -> Varchar,
        #[max_length = 100]
        firmware -> Varchar,
        #[max_length = 10]
        region -> Varchar,
        #[max_length = 10]
        mac_version -> Varchar,
        #[max_length = 20]
        reg_params_revision -> Varchar,
        #[max_length = 100]
        adr_algorithm_id -> Varchar,
        #[max_length = 20]
        payload_codec_runtime -> Varchar,
        payload_codec_script -> Text,
        uplink_interval -> Int4,
        device_status_req_interval -> Int4,
        flush_queue_on_activate -> Bool,
        supports_otaa -> Bool,
        supports_class_b -> Bool,
        supports_class_c -> Bool,
        class_b_timeout -> Int4,
        class_b_ping_slot_nb_k -> Int4,
        class_b_ping_slot_dr -> Int2,
        class_b_ping_slot_freq -> Int8,
        class_c_timeout -> Int4,
        abp_rx1_delay -> Int2,
        abp_rx1_dr_offset -> Int2,
        abp_rx2_dr -> Int2,
        abp_rx2_freq -> Int8,
        tags -> Jsonb,
        measurements -> Jsonb,
        auto_detect_measurements -> Bool,
    }
}

diesel::table! {
    device_queue_item (id) {
        id -> Uuid,
        dev_eui -> Bytea,
        created_at -> Timestamptz,
        f_port -> Int2,
        confirmed -> Bool,
        data -> Bytea,
        is_pending -> Bool,
        f_cnt_down -> Nullable<Int8>,
        timeout_after -> Nullable<Timestamptz>,
        is_encrypted -> Bool,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    door_alarm_date_time (id) {
        id -> Int4,
        alarm_id -> Int4,
        alarm_day -> Int4,
        start_time -> Float8,
        end_time -> Float8,
    }
}

diesel::table! {
    door_time_alarm (id) {
        id -> Int4,
        #[max_length = 30]
        dev_eui -> Nullable<Varchar>,
        sms -> Nullable<Bool>,
        email -> Nullable<Bool>,
        notification -> Nullable<Bool>,
        submission_time -> Nullable<Timestamp>,
        is_active -> Nullable<Bool>,
        time -> Nullable<Int8>,
        user_id -> Nullable<Array<Nullable<Int8>>>,
        organization_id -> Nullable<Int4>,
    }
}

diesel::table! {
    em400mud (id) {
        id -> Int4,
        #[max_length = 20]
        dev_eui -> Varchar,
        device_type_id -> Int4,
        org_id -> Int4,
        submission_date -> Nullable<Timestamp>,
        distance -> Nullable<Int4>,
        #[max_length = 20]
        position -> Nullable<Varchar>,
        air_temperature -> Nullable<Numeric>,
        batv -> Nullable<Int4>,
    }
}

diesel::table! {
    fuota_deployment (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        #[max_length = 100]
        name -> Varchar,
        application_id -> Uuid,
        device_profile_id -> Uuid,
        multicast_addr -> Bytea,
        multicast_key -> Bytea,
        #[max_length = 1]
        multicast_group_type -> Bpchar,
        #[max_length = 20]
        multicast_class_c_scheduling_type -> Varchar,
        multicast_dr -> Int2,
        multicast_class_b_ping_slot_nb_k -> Int2,
        multicast_frequency -> Int8,
        multicast_timeout -> Int2,
        multicast_session_start -> Nullable<Timestamptz>,
        multicast_session_end -> Nullable<Timestamptz>,
        unicast_max_retry_count -> Int2,
        fragmentation_fragment_size -> Int2,
        fragmentation_redundancy_percentage -> Int2,
        fragmentation_session_index -> Int2,
        fragmentation_matrix -> Int2,
        fragmentation_block_ack_delay -> Int2,
        fragmentation_descriptor -> Bytea,
        #[max_length = 20]
        request_fragmentation_session_status -> Varchar,
        payload -> Bytea,
        on_complete_set_device_tags -> Jsonb,
    }
}

diesel::table! {
    fuota_deployment_device (fuota_deployment_id, dev_eui) {
        fuota_deployment_id -> Uuid,
        dev_eui -> Bytea,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        mc_group_setup_completed_at -> Nullable<Timestamptz>,
        mc_session_completed_at -> Nullable<Timestamptz>,
        frag_session_setup_completed_at -> Nullable<Timestamptz>,
        frag_status_completed_at -> Nullable<Timestamptz>,
        error_msg -> Text,
    }
}

diesel::table! {
    fuota_deployment_gateway (fuota_deployment_id, gateway_id) {
        fuota_deployment_id -> Uuid,
        gateway_id -> Bytea,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    fuota_deployment_job (fuota_deployment_id, job) {
        fuota_deployment_id -> Uuid,
        #[max_length = 20]
        job -> Varchar,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        max_retry_count -> Int2,
        attempt_count -> Int2,
        scheduler_run_after -> Timestamptz,
        warning_msg -> Text,
        error_msg -> Text,
    }
}

diesel::table! {
    gateway (gateway_id) {
        gateway_id -> Bytea,
        tenant_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_seen_at -> Nullable<Timestamptz>,
        #[max_length = 100]
        name -> Varchar,
        description -> Text,
        latitude -> Float8,
        longitude -> Float8,
        altitude -> Float4,
        stats_interval_secs -> Int4,
        tls_certificate -> Nullable<Bytea>,
        tags -> Jsonb,
        properties -> Jsonb,
    }
}

diesel::table! {
    ltc2lb (id) {
        id -> Int4,
        #[max_length = 255]
        dev_eui -> Varchar,
        temperature1 -> Nullable<Numeric>,
        temperature2 -> Nullable<Numeric>,
        batv -> Nullable<Numeric>,
        org_id -> Nullable<Int4>,
        device_type_id -> Nullable<Int4>,
        submission_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    multicast_group (id) {
        id -> Uuid,
        application_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 10]
        region -> Varchar,
        mc_addr -> Bytea,
        mc_nwk_s_key -> Bytea,
        mc_app_s_key -> Bytea,
        f_cnt -> Int8,
        #[max_length = 1]
        group_type -> Bpchar,
        dr -> Int2,
        frequency -> Int8,
        class_b_ping_slot_nb_k -> Int2,
        #[max_length = 20]
        class_c_scheduling_type -> Varchar,
    }
}

diesel::table! {
    multicast_group_device (multicast_group_id, dev_eui) {
        multicast_group_id -> Uuid,
        dev_eui -> Bytea,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    multicast_group_gateway (multicast_group_id, gateway_id) {
        multicast_group_id -> Uuid,
        gateway_id -> Bytea,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    multicast_group_queue_item (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        scheduler_run_after -> Timestamptz,
        multicast_group_id -> Uuid,
        gateway_id -> Bytea,
        f_cnt -> Int8,
        f_port -> Int2,
        data -> Bytea,
        emit_at_time_since_gps_epoch -> Nullable<Int8>,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    relay_device (relay_dev_eui, dev_eui) {
        relay_dev_eui -> Bytea,
        dev_eui -> Bytea,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    relay_gateway (tenant_id, relay_id) {
        tenant_id -> Uuid,
        relay_id -> Bytea,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_seen_at -> Nullable<Timestamptz>,
        #[max_length = 100]
        name -> Varchar,
        description -> Text,
        stats_interval_secs -> Int4,
        #[max_length = 100]
        region_config_id -> Varchar,
    }
}

diesel::table! {
    tenant (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 100]
        name -> Varchar,
        description -> Text,
        can_have_gateways -> Bool,
        max_device_count -> Int4,
        max_gateway_count -> Int4,
        private_gateways_up -> Bool,
        private_gateways_down -> Bool,
        tags -> Jsonb,
        sms_count -> Nullable<Int4>,
        license -> Nullable<Bool>,
        pro_license -> Nullable<Bool>,
        kitchen_management_license -> Nullable<Bool>,
    }
}

diesel::table! {
    tenant_user (tenant_id, user_id) {
        tenant_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        is_admin -> Bool,
        is_device_admin -> Bool,
        is_gateway_admin -> Bool,
        is_visible -> Nullable<Bool>,
    }
}

diesel::table! {
    uc300 (id) {
        id -> Int4,
        #[max_length = 40]
        dev_eui -> Varchar,
        device_type_id -> Nullable<Int4>,
        org_id -> Nullable<Int4>,
        #[max_length = 20]
        adc_1 -> Nullable<Varchar>,
        #[max_length = 20]
        adc_2 -> Nullable<Varchar>,
        #[max_length = 20]
        adv_1 -> Nullable<Varchar>,
        #[max_length = 20]
        gpio_in_1 -> Nullable<Varchar>,
        #[max_length = 20]
        gpio_in_2 -> Nullable<Varchar>,
        #[max_length = 20]
        gpio_in_3 -> Nullable<Varchar>,
        #[max_length = 20]
        gpio_in_4 -> Nullable<Varchar>,
        #[max_length = 20]
        gpio_out_1 -> Nullable<Varchar>,
        #[max_length = 20]
        gpio_out_2 -> Nullable<Varchar>,
        submission_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user (id) {
        id -> Uuid,
        external_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        is_admin -> Bool,
        is_active -> Bool,
        email -> Text,
        email_verified -> Bool,
        #[max_length = 200]
        password_hash -> Varchar,
        note -> Text,
        #[max_length = 250]
        android_key -> Nullable<Varchar>,
        #[max_length = 50]
        phone_number -> Varchar,
        name -> Nullable<Text>,
        username -> Nullable<Text>,
        zone_id_list -> Nullable<Array<Nullable<Int8>>>,
        training -> Bool,
        #[max_length = 250]
        expo_key -> Nullable<Varchar>,
        #[max_length = 250]
        web_key -> Nullable<Varchar>,
    }
}

diesel::table! {
    ws522 (id) {
        id -> Int4,
        #[max_length = 40]
        dev_eui -> Nullable<Varchar>,
        device_type_id -> Nullable<Int4>,
        org_id -> Nullable<Int4>,
        current -> Nullable<Numeric>,
        factor -> Nullable<Numeric>,
        power -> Nullable<Numeric>,
        voltage -> Nullable<Numeric>,
        power_sum -> Nullable<Numeric>,
        submission_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    ws558 (id) {
        id -> Int4,
        #[max_length = 20]
        dev_eui -> Varchar,
        device_type_id -> Int4,
        org_id -> Int4,
        submission_date -> Nullable<Timestamp>,
        switch1 -> Nullable<Int4>,
        switch2 -> Nullable<Int4>,
        switch3 -> Nullable<Int4>,
        switch4 -> Nullable<Int4>,
        switch5 -> Nullable<Int4>,
        switch6 -> Nullable<Int4>,
        switch7 -> Nullable<Int4>,
        switch8 -> Nullable<Int4>,
        power -> Nullable<Numeric>,
        power_consumption -> Nullable<Numeric>,
        factor -> Nullable<Numeric>,
        current -> Nullable<Numeric>,
        voltage -> Nullable<Numeric>,
    }
}

diesel::table! {
    zone (zone_id) {
        zone_id -> Int4,
        #[max_length = 100]
        zone_name -> Nullable<Varchar>,
        org_id -> Nullable<Int8>,
        devices -> Nullable<Array<Nullable<Text>>>,
        zone_order -> Nullable<Int8>,
        content_type -> Nullable<Int8>,
    }
}

diesel::joinable!(api_key -> tenant (tenant_id));
diesel::joinable!(application -> tenant (tenant_id));
diesel::joinable!(application_integration -> application (application_id));
diesel::joinable!(device -> application (application_id));
diesel::joinable!(device -> device_profile (device_profile_id));
diesel::joinable!(device_keys -> device (dev_eui));
diesel::joinable!(device_profile -> tenant (tenant_id));
diesel::joinable!(device_queue_item -> device (dev_eui));
diesel::joinable!(fuota_deployment -> application (application_id));
diesel::joinable!(fuota_deployment -> device_profile (device_profile_id));
diesel::joinable!(fuota_deployment_device -> device (dev_eui));
diesel::joinable!(fuota_deployment_device -> fuota_deployment (fuota_deployment_id));
diesel::joinable!(fuota_deployment_gateway -> fuota_deployment (fuota_deployment_id));
diesel::joinable!(fuota_deployment_gateway -> gateway (gateway_id));
diesel::joinable!(fuota_deployment_job -> fuota_deployment (fuota_deployment_id));
diesel::joinable!(gateway -> tenant (tenant_id));
diesel::joinable!(multicast_group -> application (application_id));
diesel::joinable!(multicast_group_device -> device (dev_eui));
diesel::joinable!(multicast_group_device -> multicast_group (multicast_group_id));
diesel::joinable!(multicast_group_gateway -> gateway (gateway_id));
diesel::joinable!(multicast_group_gateway -> multicast_group (multicast_group_id));
diesel::joinable!(multicast_group_queue_item -> gateway (gateway_id));
diesel::joinable!(multicast_group_queue_item -> multicast_group (multicast_group_id));
diesel::joinable!(relay_gateway -> tenant (tenant_id));
diesel::joinable!(tenant_user -> tenant (tenant_id));
diesel::joinable!(tenant_user -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    alarm,
    alarm_audit_log,
    alarm_automation_rules,
    alarm_date_time,
    am103,
    api_key,
    application,
    application_integration,
    automation_rules,
    dds45lb,
    device,
    device_data_2025,
    device_data_latest,
    device_keys,
    device_profile,
    device_profile_template,
    device_queue_item,
    door_alarm_date_time,
    door_time_alarm,
    em400mud,
    fuota_deployment,
    fuota_deployment_device,
    fuota_deployment_gateway,
    fuota_deployment_job,
    gateway,
    ltc2lb,
    multicast_group,
    multicast_group_device,
    multicast_group_gateway,
    multicast_group_queue_item,
    relay_device,
    relay_gateway,
    tenant,
    tenant_user,
    uc300,
    user,
    ws522,
    ws558,
    zone,
);
